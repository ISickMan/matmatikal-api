use crate::{schema::users::dsl::*, DbPool};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use jsonwebtoken::{
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use reqwest::Client;
use serde::Deserialize;
const CLIENT_ID: &'static str =
    "794981933073-c59qh87r995625mjrk4iph5m89cd9s03.apps.googleusercontent.com";

#[derive(Debug, Deserialize)]
struct GoogleLoginJwt {
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
        .json().await
        .ok()
}

#[post("/google-login")]
pub async fn google_login(
    pool: web::Data<DbPool>,
    req_body: String,
) -> actix_web::Result<impl Responder> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&["https://accounts.google.com"]); // Issuer for Google One Tap
    validation.set_audience(&[CLIENT_ID]);

    let credential = req_body;
    let header = jsonwebtoken::decode_header(&credential).unwrap();

    let Some(kid) = header.kid else {
        return panic!("Token doesn't have a `kid` header field");
    };

    let jwks: JwkSet = fetch_google_public_keys().await.expect("UH");
    let Some(jwk) = jwks.find(&kid) else {
        return panic!("No matching JWK found for the given kid");
    };

    let decoding_key = match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => DecodingKey::from_rsa_components(&rsa.n, &rsa.e).unwrap(),
        _ => unreachable!("algorithm should be a RSA in this example"),
    };

    let credential_data =
        jsonwebtoken::decode::<GoogleLoginJwt>(&credential, &decoding_key, &validation)
            .unwrap()
            .claims;
    println!("body: {:#?}", credential_data);

    let uname = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        users
            .filter(email.eq(&credential_data.email))
            .select(username)
            .first::<String>(&mut conn)
    })
    .await?;

    let Ok(uname) = uname else {
        // user doesnt exist
        return Ok(HttpResponse::NotFound().json("doesn't exist"));
    };

    Ok(HttpResponse::Ok().body(uname))
}
