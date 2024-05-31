use std::collections::HashMap;

use actix_web::{delete, get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use iter_tools::Itertools;
use serde_json::Value;
use ts_rs::TS;
use crate::auth::login::UserClaims;
use crate::schema::{sketch_groups, sketches};
use crate::schema::sketches::dsl::*;
use diesel::prelude::*;
use crate::DbPool;

#[derive(Queryable, Identifiable, Selectable, Associations, Debug)]
// #[diesel(belongs_to(User, foreign_key= creator_id))]
#[diesel(belongs_to(SketchGroupDb, foreign_key = sketch_group))]
#[diesel(table_name = sketches)]
pub struct SketchDb {
    id: i32,
    name: String,
    creator_id: i32,
    data: String,
    creation_time: DateTime<Utc>,
    sketch_group: Option<i32>,
}

#[derive(Queryable, Identifiable, Debug)]
#[diesel(table_name = sketch_groups)]
pub struct SketchGroupDb {
    id: i32,
    name: String,
    creator_id: i32,
}

#[derive(Serialize, Debug, TS)]
// the way it gets to the client
#[ts(export)]
pub struct Sketch {
    id: i32,
    name: String,
    #[ts(type = "any[]")]
    data: Vec<Value>,
    creator: String,
    #[ts(type="number")]
    creation_time_unix: u64
}


#[get("/explore")]
pub async fn explore(
    pool: web::Data<DbPool>,
) -> actix_web::Result<HttpResponse> {
    use crate::schema::users::dsl::*;
    use crate::schema::sketch_groups::dsl::*;

    let sketches_with_groups: Result<Vec<(SketchDb, String, String)>, diesel::result::Error> = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        sketches
            .inner_join(users)
            .inner_join(sketch_groups)
            .select((SketchDb::as_select(), username, name))
            .load::<(SketchDb, String, String)>(&mut conn)
    }).await?;

    Ok(match sketches_with_groups {
        Ok(results) => {
            let mut grouped_sketches: HashMap<String, Vec<Sketch>> = HashMap::new();

            for (sketch_db, creator, group_name) in results {
                let sketch = Sketch {
                    id: sketch_db.id,
                    name: sketch_db.name,
                    data: serde_json::from_str(&sketch_db.data).unwrap(),
                    creator,
                    creation_time_unix: sketch_db.creation_time.timestamp() as u64,
                };

                grouped_sketches.entry(group_name)
                    .or_insert_with(Vec::new)
                    .push(sketch);
            }

            HttpResponse::Ok().json(grouped_sketches)
        }
        Err(e) => {
            eprintln!("Failed to load sketches {}", e);
            HttpResponse::NotFound().json(e.to_string())
        }
    })
}

#[delete("/delete/{id}")]
pub async fn delete_sketch(
    claims: UserClaims,
    pool: web::Data<DbPool>,
    sketch_id: web::Path<i32>
) -> actix_web::Result<HttpResponse> {
    let sketch_id = sketch_id.into_inner();

    if &claims.username != "admin" {
       return Ok(HttpResponse::Unauthorized().into()); 
    }

    let result: Result<usize, diesel::result::Error> = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        diesel::delete(sketches.filter(id.eq(sketch_id)))
            .execute(&mut conn)
    }).await?;

    Ok(match result {
        Ok(count) if count > 0 => HttpResponse::Ok().json("Sketch deleted successfully"),
        Ok(_) => HttpResponse::NotFound().json("Sketch not found"),
        Err(e) => {
            eprintln!("Failed to delete sketch {}", e);
            HttpResponse::InternalServerError().json(e.to_string())
        }
    })
}