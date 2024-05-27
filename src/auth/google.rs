use std::collections::HashMap;

use crate::{auth::login::jwt_response, schema::users::dsl::*, DbPool};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use jsonwebtoken::{
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

lazy_static::lazy_static! {
    static ref GOOGLE_CLIENT_ID: String =
        dotenvy::var("GOOGLE_CLIENT_ID").unwrap();

    static ref GOOGLE_API_KEY: String =
        dotenvy::var("GOOGLE_API_KEY").unwrap();

     static ref GOOGLE_CLIENT_SECRET: String =
        dotenvy::var("GOOGLE_CLIENT_SECRET").unwrap();


}

#[derive(Debug, Deserialize)]
struct GoogleLoginJwt {
    sub: String,
    email: String,
    name: String,
}
async fn fetch_google_public_keys() -> Option<JwkSet> {
    let client = Client::new();
    client
        .get("https://www.googleapis.com/oauth2/v3/certs")
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()
}

#[derive(Deserialize)]
struct BirthdayParams {
    google_id: String,
    code: String,
}

#[derive(Serialize, Deserialize)]
struct Date {
    year: u16,
    month: u8,
    day: u8,
}


#[get("/birthday")]
pub async fn get_birthday(params: web::Query<BirthdayParams>) -> impl Responder {
    let access_token = &params.code;
    let url = format!(
        "https://people.googleapis.com/v1/people/{}?personFields=birthdays&key={}&access_token={}",
        params.google_id, &*GOOGLE_API_KEY, access_token
    );

    HttpResponse::Ok().json(match fetch_person(&url).await {
        Ok(json) => match extract_date(&json) {
            Some(date) => json!(date),
            None => json!("No birthday found"),
        },
        Err(err) => json!(format!("Error: {:?}", err)),
    })
}

#[derive(Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    scope: String,
    token_type: String,
    refresh_token: Option<String>,
}

fn extract_date(json: &Value) -> Option<Date> {
    println!("{}", json);
    json.get("birthdays")
        .and_then(|birthdays| birthdays.as_array())
        .and_then(|array| array.get(0))
        .and_then(|bday| bday.get("date"))
        .and_then(|date| serde_json::from_value(date.clone()).ok())
}

async fn fetch_person(url: &str) -> Result<Value, reqwest::Error> {
    let client = Client::new();
    let response = client.get(url).send().await?;
    let json = response.json::<Value>().await?;
    Ok(json)
}

#[post("/google-login")]
pub async fn google_login(
    pool: web::Data<DbPool>,
    req_body: String,
) -> actix_web::Result<impl Responder> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://accounts.google.com"]); // Issuer for Google One Tap
    validation.set_audience(&[&*GOOGLE_CLIENT_ID]);

    let credential = req_body;
    let header = jsonwebtoken::decode_header(&credential).unwrap();

    let Some(kid) = header.kid else {
        panic!("Token doesn't have a `kid` header field");
    };

    let jwks: JwkSet = fetch_google_public_keys().await.expect("UH");
    let Some(jwk) = jwks.find(&kid) else {
        panic!("No matching JWK found for the given kid");
    };

    let decoding_key = match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap(),
        _ => unreachable!("algorithm should be a RSA in this example"),
    };

    let credential_data =
        jsonwebtoken::decode::<GoogleLoginJwt>(&credential, &decoding_key, &validation)
            .unwrap()
            .claims;
    let eml = credential_data.email.clone();
    println!("body: {:#?}", credential_data);

    let uname = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        users
            .filter(email.eq(&credential_data.email))
            .select((username, id))
            .first::<(String, i32)>(&mut conn)
    })
    .await?;

    let Ok((uname, uid)) = uname else {
        // user doesnt exist
        let gid = credential_data.sub;
        return Ok(HttpResponse::NotFound().json(json!({"gid": gid, "email": eml})));
    };

    Ok(jwt_response(uname, uid))
}
