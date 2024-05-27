use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::pg::expression::extensions;
use serde::Deserialize;
use ts_rs::TS;

use crate::auth::login::UserClaims;
use crate::schema::sketches;
use crate::schema::sketches::dsl::*;
use diesel::prelude::*;
use crate::DbPool;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UploadWeb {}

#[derive(Insertable)]
#[diesel(table_name = sketches)]
pub struct InsertSketch {
    creator_id: i32,
    creation_time: DateTime<Utc>,
}

#[post("/upload")]
pub async fn upload(
    req: HttpRequest,
    claims: UserClaims,
    pool: web::Data<DbPool>,
    data: web::Json<UploadWeb>
) -> actix_web::Result<impl Responder> {
    println!("claims: {:#?}", claims);
    let new_sketch = InsertSketch {
        creation_time: Utc::now(),
        creator_id: claims.id,
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
        Err(e) =>{
            eprintln!("Failed to add sketch {}", e);
             HttpResponse::InternalServerError().into()
        }
    })
}
