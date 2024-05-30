use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::pg::expression::extensions;
use serde::Deserialize;
use serde_json::Value;
use ts_rs::TS;

use crate::auth::login::UserClaims;
use crate::schema::sketches;
use crate::schema::sketches::dsl::*;
use crate::DbPool;
use diesel::prelude::*;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UploadWeb {
    name: String,
    #[ts(type = "any[]")]
    data: Vec<Value>,
}

#[derive(Insertable)]
#[diesel(table_name = sketches)]
pub struct InsertSketch {
    name: String,
    data: String,
    creator_id: i32,
    creation_time: DateTime<Utc>,
}

#[post("/upload")]
pub async fn upload(
    claims: UserClaims,
    pool: web::Data<DbPool>,
    sketch_data: web::Json<UploadWeb>,
) -> actix_web::Result<impl Responder> {
    println!("claims: {:#?}", claims);
    let new_sketch = InsertSketch {
        name: sketch_data.name.clone(),
        creation_time: Utc::now(),
        creator_id: claims.id,
        data: serde_json::to_string(&sketch_data.data).unwrap(),
    };
    let sketch_id = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        diesel::insert_into(sketches::table)
            .values(new_sketch)
            .returning(id)
            .get_result::<i32>(&mut conn)
    })
    .await?;

    Ok(match sketch_id {
        Ok(k) => HttpResponse::Ok().json(k),
        Err(e) => {
            eprintln!("Failed to add sketch {}", e);
            HttpResponse::InternalServerError().into()
        }
    })
}
    