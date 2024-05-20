use actix_web::{error, post, web, Responder};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::{prelude::Insertable, RunQueryDsl};
use sha2::{Digest, Sha256};

use crate::DbPool;

use super::users::UserRegisterWeb;
use crate::schema::users;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct InsertUser<'a> {
    username: &'a str,
    google_id: Option<&'a str>,
    pass_hash: &'a str,
    birthday: NaiveDate,
    creation_time: DateTime<Utc>,
}

#[post("/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    user: web::Json<UserRegisterWeb>,
) -> actix_web::Result<impl Responder> {
    web::block(move || {
        // Obtaining a connection from the pool is also a potentially blocking operation.
        // So, it should be called within the `web::block` closure, as well.
        let mut conn = pool.get().expect("couldn't get db connection from pool");
        let pass_hash = hex::encode(Sha256::digest(user.password.as_bytes()));
        let (year, month, date) = user.birthday_date_ymd;

        let new_user = InsertUser {
            username: &user.username,
            google_id: user.google_id.as_deref(),
            pass_hash: &pass_hash,
            birthday: NaiveDate::from_ymd_opt(year, month, date).unwrap()
            /* .map_err(error::ErrorInternalServerError) */,
            creation_time: Utc::now(),
        };

        diesel::insert_into(users::table).values(new_user).execute(&mut conn)
        .expect("Error saving new post");
    })
    .await?;
    // .map_err(error::ErrorInternalServerError)?;

    Ok("OK")
}
