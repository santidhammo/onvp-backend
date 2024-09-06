use dotenv::dotenv;
use onvp_backend::api;
use std::error::Error;
#[actix_web::main]
async fn main() -> Result<(), impl Error> {
    env_logger::init();
    dotenv().ok();
    api::run_api_server().await
}
