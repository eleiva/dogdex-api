mod errors;
mod models;
mod schema;

use actix_files::Files;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::RunQueryDsl;
use dogdex_api::{init, AppSettings, DBPool};
use errors::UserError;
use handlebars::Handlebars;
use log::error;
use models::Dog;
use schema::dogs::dsl::*;
use serde::Deserialize;
use validator::Validate;
use validator_derive::Validate;

async fn index(hb: web::Data<Handlebars<'_>>) -> HttpResponse {
    let body: String = hb.render("landing", &{}).unwrap();
    HttpResponse::Ok().body(body)
}

#[derive(Deserialize, Validate)]
pub struct DogEndpointPath {
    #[validate(range(min = 1, max = 150))]
    id: i32,
}

pub async fn get_dog(
    pool: web::Data<DBPool>,
    dog_id: web::Path<DogEndpointPath>,
) -> Result<HttpResponse, Error> {

    dog_id
        .validate()
        .map_err(|_| {
            error!("Validation error");
            UserError::ValidationError
        })?;

    let dog_data = web::block(move || {
        dogs.filter(id.eq(dog_id.into_inner().id))
            .first::<Dog>(&mut pool.get().unwrap())
    })
    .await
    .map_err(|_| {
        error!("Blocking Thread Pool Error");
        UserError::UnexpectedError
    })?
    .map_err(|_| {
        error!("Failed to get DB connection from pool");
        UserError::DBPoolGetError
    })?;

    Ok(HttpResponse::Ok().json(dog_data))
}

pub async fn get_dogs(pool: web::Data<DBPool>) -> Result<HttpResponse, Error> {
    let dogs_data: Vec<Dog> =
        web::block(move || dogs.limit(100).load::<Dog>(&mut pool.get().unwrap()))
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
            .configure(api_config)
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn api_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/dogs", web::get().to(get_dogs))
            .route("/dogs/{id}", web::get().to(get_dog)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::App;
    use actix_web::{test, web};
    use dogdex_api::{setup_database, DBPool};

    #[actix_rt::test]
    async fn test_dogs_endpoint_get() {
        let pool: DBPool = setup_database(true);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .configure(api_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/dogs").to_request();
        let resp: actix_web::dev::ServiceResponse = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn it_test_get_dog_endpoint() {
        let pool: DBPool = setup_database(true);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .configure(api_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/dogs/1").to_request();
        let resp: actix_web::dev::ServiceResponse = test::call_service(&mut app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn it_test_validation_at_get_dog() {
        let pool: DBPool = setup_database(true);

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .configure(api_config),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/dogs/800").to_request();
        let resp: actix_web::dev::ServiceResponse = test::call_service(&mut app, req).await;

        assert_eq!(resp.status(), 400);
    }
}
