use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::pg::expression::extensions;
use serde::Deserialize;
use serde_json::Value;
use ts_rs::TS;

use crate::auth::login::UserClaims;
use crate::schema::sketches::dsl::*;
use crate::schema::{sketch_groups, sketches};
use crate::DbPool;
use diesel::prelude::*;

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UploadWeb {
    name: String,
    #[ts(type = "any[]")]
    data: Vec<Value>,
    group: String,
}

#[derive(Insertable)]
#[diesel(table_name = sketches)]
pub struct InsertSketch {
    name: String,
    data: String,
    creator_id: i32,
    creation_time: DateTime<Utc>,
    sketch_group: Option<i32>,
}

#[post("/upload")]
pub async fn upload(
    claims: UserClaims,
    pool: web::Data<DbPool>,
    sketch_data: web::Json<UploadWeb>,
) -> actix_web::Result<impl Responder> {
    let user_id = claims.id;
    
    println!("claims: {:#?}", claims);
    let sketch_id = web::block(move || {
        let group_name = &sketch_data.group;
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        // Check if the group exists
        let group_id = sketch_groups::table
            .filter(sketch_groups::name.eq(&group_name))
            .select(sketch_groups::id)
            .first::<i32>(&mut conn)
            .optional()?
            .unwrap_or_else(|| {
                // If the group doesn't exist, create it
                diesel::insert_into(sketch_groups::table)
                    .values((
                        sketch_groups::name.eq(&group_name),
                        sketch_groups::creator_id.eq(user_id),
                    ))
                    .returning(sketch_groups::id)
                    .get_result(&mut conn)
                    .expect("Error creating new group")
            });

        let new_sketch = InsertSketch {
            name: sketch_data.name.clone(),
            creation_time: Utc::now(),
            creator_id: claims.id,
            data: serde_json::to_string(&sketch_data.data).unwrap(),
            sketch_group: Some(group_id),
        };

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
