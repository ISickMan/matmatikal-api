use actix_web::{post, web, HttpResponse, Responder};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{schema::users::dsl::*, DbPool};
use diesel::prelude::*;

use super::{register::password_hash, users::UserLoginWeb};

lazy_static::lazy_static! {
    pub static ref JWT_ENCODING_KEY : EncodingKey = EncodingKey::from_secret(dotenvy::var(
        "JWT_ENCODING_KEY"
    ).expect("MISSING JWT TOKEN ENCODER").as_bytes());
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    // aud: String,         // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    // iat: usize,          // Optional. Issued at (as UTC timestamp)
    // iss: String,         // Optional. Issuer
    // nbf: usize,          // Optional. Not Before (as UTC timestamp)
    // sub: String,         // Optional. Subject (whom token refers to)
    username: String,
}

#[post("/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    creds: web::Json<UserLoginWeb>,
) -> actix_web::Result<impl Responder> {
    login_internal(pool, creds.0).await
}

// pub async fn get_user_by_email(con: &mut PgConnection, email: &str) -> {

// }

pub(crate) async fn login_internal(
    pool: web::Data<DbPool>,
    creds: UserLoginWeb,
) -> actix_web::Result<HttpResponse> {
    let uname = web::block(move || {
        let hash = password_hash(&creds.password);
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        users
            .filter(email.eq(&creds.email).and(pass_hash.eq(&hash)))
            .select(username)
            .first::<String>(&mut conn)
    })
    .await?;


    // get current time in unix seconds
    let expiration = chrono::Utc::now().timestamp() as usize + 60 * 60; // 1 hour expiration

    let Ok(uname) = uname else {
        return Ok(HttpResponse::NotFound().json("Wrong credentials"));
    };

    let jwt = encode(
        &Header::default(),
        &Claims {
            exp: expiration,
            username: uname,
        },
        &JWT_ENCODING_KEY,
    )
    .unwrap();

    Ok(HttpResponse::Ok().json(jwt))
}
