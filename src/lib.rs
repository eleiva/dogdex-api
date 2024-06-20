mod errors;
mod models;
mod schema;

use actix_web::web::{self, Data};
use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};
use dotenv::dotenv;
use handlebars::{DirectorySourceOptions, Handlebars};
use std::env;

pub struct AppSettings<'a> {
    pub handlebars: Data<Handlebars<'a>>,
    pub pool: r2d2::Pool<ConnectionManager<SqliteConnection>>,
}

pub fn init() -> AppSettings<'static> {
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
