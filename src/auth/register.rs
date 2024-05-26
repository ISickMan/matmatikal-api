use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::{prelude::Insertable, result::{DatabaseErrorKind, Error}, RunQueryDsl};
use sha2::{Digest, Sha256};

use crate::{auth::{login, users::UserLoginWeb}, DbPool};

use super::users::UserRegisterWeb;
use crate::schema::users;

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct InsertUser<'a> {
    username: &'a str,
    email: &'a str,
    google_id: Option<&'a str>,
    pass_hash: &'a str,
    birthday: NaiveDate,
    creation_time: DateTime<Utc>,
}

pub fn password_hash(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}


#[post("/register")]
pub async fn register(
    pool: web::Data<DbPool>,
    user: web::Json<UserRegisterWeb>,
) -> actix_web::Result<HttpResponse> {
    register_internal(pool, user.0).await
}


pub(crate) async fn register_internal(
    pool: web::Data<DbPool>,
    user: UserRegisterWeb,
) -> actix_web::Result<HttpResponse> {
    let pool2 = pool.clone();

    let (email, password) = (user.email.clone(), user.password.clone());

    if let Some(err) = web::block(move || {
        let pass_hash = password_hash(&user.password);
        let (year, month, date) = user.birthday_date_ymd;
        
        let new_user = InsertUser {
            username: &user.username,
            email: &user.email,
            google_id: user.google_id.as_deref(),
            pass_hash: &pass_hash,
            birthday: NaiveDate::from_ymd_opt(year, month, date).unwrap()
            /* .map_err(error::ErrorInternalServerError) */,
            creation_time: Utc::now(),
        };
        
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        if let Err(e) = diesel::insert_into(users::table).values(new_user).execute(&mut conn) {

        //    handle unique violation
        
            if let Error::DatabaseError(DatabaseErrorKind::UniqueViolation, e) = e {
                let message = e.message();
                let column = if message.find("email").is_some() {
                    "email"
                }else {"username"};
                
                return Some(
                    "Duplicate ".to_string() + column
                );
            }
        }
    

        None
    })
    .await? {
        return Ok(HttpResponse::Conflict().json(
                err
            ));
 
        }
    // .map_err(error::ErrorInternalServerError)?;

    crate::auth::login::login_internal(pool2, UserLoginWeb {
        email,
        password,
    }).await
}