use std::{env, error};

use actix_files::Files;
use actix_web::{
    web::{self, Data},
    App, Error, HttpResponse, HttpServer,
};
use diesel::{
    r2d2::{self, ConnectionManager},
    RunQueryDsl, SqliteConnection,
};
use dogdex_api::{errors::UserError, models::Dog};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
mod models;
mod schema;
mod errors;

use schema::dogs::dsl::*;
use diesel::prelude::*;
use log::{error, info, warn};



async fn index(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let body: String = hb.render("landing", &{}).unwrap();

    HttpResponse::Ok().body(body)
}

// async fn get_dogs(
//     pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>,
// ) -> Result<HttpResponse, Error> {

//     let dogs_data    = web::block(move || {
//         // Usa la conexión obtenida
//         dogs.limit(100).load::<Dog>(&mut pool.get().unwrap()).map_err(|e| e.to_string())
//     })
//     .await
//     .map_err(|_| HttpResponse::InternalServerError().finish());

//     match dogs_data {
//         Ok(dogs) => Ok(HttpResponse::Ok().json(dogs)),
//         Err(_) => Ok(HttpResponse::InternalServerError().finish()),  // Manejo de errores de Diesel
//     }
// }

async fn get_dogs(pool: web::Data<r2d2::Pool<ConnectionManager<SqliteConnection>>>) -> Result<HttpResponse, Error> {
    let mut connection = pool.get().expect("Can't get db connection from pool");
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

struct AppSettings<'a> {
    handlebars: Data<Handlebars<'a>>,
    pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
}

fn init() -> AppSettings<'static> {
    dotenv().ok();

    let mut handlebars: Handlebars = Handlebars::new();

    handlebars
        .register_templates_directory(
            "./static",
            DirectorySourceOptions {
                tpl_extension: ".html".to_owned(),
                hidden: false,
                temporary: false,
            },
        )
        .unwrap();

    let database_url: String = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager: ConnectionManager<SqliteConnection> =
        ConnectionManager::<SqliteConnection>::new(&database_url);

    let pool: r2d2::Pool<ConnectionManager<SqliteConnection>> = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connection pool.");

    let app_settings: AppSettings = AppSettings {
        handlebars: web::Data::new(handlebars),
        pool: pool,
    };

    app_settings
}
