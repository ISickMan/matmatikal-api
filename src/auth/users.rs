use serde::Deserialize;
use ts_rs::TS;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UserRegisterWeb {
    pub username: String,
    pub email: String,
    pub google_id: Option<String>,
    pub password: String,
    pub birthday_date_ymd: (i32, u32, u32),
    pub grade: u8,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UserLoginWeb {
    pub email: String,
    // pub google_id: Option<String>,
    pub password: String,
}