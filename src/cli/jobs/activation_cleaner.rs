use dotenv::dotenv;
use onvp_backend::commands;
use onvp_backend::initialize_db_pool;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv().ok();
    let pool = initialize_db_pool();

    commands::jobs::clean_late_non_activated_members(&pool)?;

    Ok(())
}
