use diesel::{associations::Identifiable, deserialize::Queryable, Selectable};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use crate::schema::users;

#[derive(Queryable, Selectable, Identifiable, Serialize, Debug, TS)]
#[diesel(table_name = users)]
#[ts(export)]
pub struct User {
    id: i32,
    pub username: String,
    pub email: String,
    pub google_id: Option<String>,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UserRegisterWeb {
    pub username: String,
    pub email: String,
    pub google_id: Option<String>,
    pub password: String,
    pub birthday_date_ymd: (i32, u32, u32),
    pub grade: i16,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UserLoginWeb {
    pub email: String,
    // pub google_id: Option<String>,
    pub password: String,
}