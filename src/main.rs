pub mod auth;

use actix_cors::Cors;
use actix_web::web;
use actix_web::{http::header, App, HttpServer};
use auth::google::{get_birthday, google_login};
use auth::login::login;
use auth::register::register;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
mod schema;

pub fn get_connection_pool() -> DbPool {
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(url);
    // Refer to the `r2d2` documentation for more methods to use
    // when building a connection pool
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool =  get_connection_pool();
    // let cors = Cors::default()
    //     .allowed_origin("https://www.rust-lang.org")
    //     .allowed_methods(vec!["GET", "POST"])
    //     .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
    //     .allowed_header(header::CONTENT_TYPE)
    //     .max_age(3600);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
        .wrap(cors)
        .app_data(web::Data::new(pool.clone()))
        .service(
            web::scope("/auth")
            .service(google_login)
            .service(login)
            .service(register)
            .service(get_birthday)
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
