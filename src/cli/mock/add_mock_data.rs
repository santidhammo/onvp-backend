use chrono::TimeDelta;
use diesel::Connection;
use dotenv::dotenv;
use onvp_backend::model::security::Role;
use onvp_backend::{dal, Error};

fn main() {
    env_logger::init();
    dotenv().ok();
    let pool = dal::initialize_db_pool();
    let mut conn = pool.get().expect("couldn't get DB connection from pool");

    let _ = conn
        .transaction::<_, Error, _>(|conn| {
            dal::mock::members::create(conn, 20, TimeDelta::days(10), Role::Member)?;
            Ok(())
        })
        .expect("couldn't initialize mock data");
}
