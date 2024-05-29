use actix_web::{get, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use serde::Serialize;
use iter_tools::Itertools;
use serde_json::Value;
use ts_rs::TS;
use crate::schema::sketches;
use crate::schema::sketches::dsl::*;
use diesel::prelude::*;
use crate::DbPool;

#[derive(Queryable, Identifiable, Selectable, Debug)]
#[diesel(belongs_to(User))]
#[diesel(table_name = sketches)]
pub struct SketchDb {
    id: i32,
    name: String,
    creator_id: i32,
    data: String,
    creation_time: DateTime<Utc>
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
    use crate::schema::users;
    use crate::schema::users::dsl::*;

    let sketches_vec: Result<Vec<(SketchDb, String)>, diesel::result::Error> = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        sketches
        // .limit(10)
        .inner_join(users::table).select((SketchDb::as_select(), username)).load::<(SketchDb, String)>(&mut conn)
    }).await?;

    Ok(match sketches_vec {
        Ok(k) => HttpResponse::Ok().json(
            k.into_iter().map(|(s, creator)| {
                Sketch {
                    id: s.id,
                    creation_time_unix: s.creation_time.timestamp() as u64,
                    name: s.name,
                    data: serde_json::from_str(&s.data).unwrap(),
                    creator,
                }
            }).collect_vec()
        ),
        Err(e) => {
            eprintln!("Failed to load sketches {}", e);
            HttpResponse::NotFound().json(e.to_string())
        }
    })
}

