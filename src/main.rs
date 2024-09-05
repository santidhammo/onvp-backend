use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use diesel::{r2d2, PgConnection};
use dotenv::dotenv;
use std::error::Error;
use std::net::Ipv4Addr;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

mod api;
mod dal;
pub(crate) mod model;
mod schema;
mod security;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();
    dotenv().ok();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            api::members::operator_check,
            api::members::list,
            api::members::setup_first_operator,
        ),
        components(
            schemas(model::members::Member),
            schemas(model::setup::FirstOperator),
        ),
        tags(
            (name = "api::members", description = "Member management endpoints")
        ),
    )]
    struct ApiDoc;

    let pool = initialize_db_pool();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope(api::members::CONTEXT)
                    .service(api::members::list)
                    .service(api::members::operator_check)
                    .service(api::members::setup_first_operator),
            )
            .service(RapiDoc::with_openapi("/docs/openapi.json", ApiDoc::openapi()).path("/docs"))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}

fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be a valid URL towards PostgreSQL database")
}
