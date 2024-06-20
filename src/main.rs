mod errors;
mod models;
mod schema;

use actix_files::Files;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::{
    r2d2::{self, ConnectionManager},
    RunQueryDsl, SqliteConnection,
};
use dogdex_api::{init, AppSettings};
use errors::UserError;
use handlebars::Handlebars;
use log::error;
use models::Dog;
use schema::dogs::dsl::*;

async fn index(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let body: String = hb.render("landing", &{}).unwrap();
    HttpResponse::Ok().body(body)
}

async fn get_dogs(
    pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
) -> Result<HttpResponse, Error> {
    let dogs_data = web::block(move || dogs.limit(100).load::<Dog>(&mut pool.get().unwrap()))
        .await
        .map_err(|_| {
            error!("Blocking Thread Pool Error");
            UserError::UnexpectedError
        })?
        .map_err(|_| {
            error!("Failed to get DB connection from pool");
            UserError::DBPoolGetError
        })?;

    Ok(HttpResponse::Ok().json(dogs_data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_settings: AppSettings<'_> = init();

    println!("Listening on port 8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_settings.handlebars.clone())
            .app_data(web::Data::new(app_settings.pool.clone()))
            .service(Files::new("/public", "static").show_files_listing())
            .service(web::scope("/api").route("/dogs", web::get().to(get_dogs)))
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
