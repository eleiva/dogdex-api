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

pub type DBPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

pub struct AppSettings<'a> {
    pub handlebars: Data<Handlebars<'a>>,
    pub pool: DBPool,
}

pub fn init() -> AppSettings<'static> {

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

    let app_settings: AppSettings = AppSettings {
        handlebars: web::Data::new(handlebars),
        pool: setup_database(false),
    };

    app_settings
}

pub fn setup_database(is_test: bool) -> DBPool {
    dotenv().ok();

    let key: String ;

    if is_test {
        key = "TEST_DATABASE_URL".to_string();
    } else {
        key = "DATABASE_URL".to_string();
    }

    let database_url: String = env::var(key).expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<SqliteConnection>::new(&database_url);

    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB connection pool.")
}
