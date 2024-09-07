use dotenv::dotenv;
use onvp_backend::{commands, dal};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    dotenv().ok();
    let pool = dal::initialize_db_pool();

    commands::jobs::clean_late_non_activated_members(pool)?;

    Ok(())
}
