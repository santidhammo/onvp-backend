use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use onvp_backend::{api, model};
use std::error::Error;
use std::net::Ipv4Addr;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();
    dotenv().ok();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            api::setup::should_setup,
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

    let pool = onvp_backend::initialize_db_pool();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope(api::members::CONTEXT)
                    .service(api::members::list)
                    .service(api::members::setup_first_operator),
            )
            .service(web::scope(api::setup::CONTEXT).service(api::setup::should_setup))
            .service(RapiDoc::with_openapi("/docs/openapi.json", ApiDoc::openapi()).path("/docs"))
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
