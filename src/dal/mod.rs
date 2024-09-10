use crate::Error;
use actix_web::web::Data;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;

pub mod members;

pub fn connect(
    pool: &Data<DbPool>,
) -> Result<PooledConnection<ConnectionManager<DbConnection>>, Error> {
    pool.get().map_err(|e| crate::Error::from(e))
}

pub type DbPool = r2d2::Pool<ConnectionManager<DbConnection>>;

#[derive(diesel::MultiConnection)]
pub enum DbConnection {
    PostgreSQL(PgConnection),
}

pub fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = ConnectionManager::<DbConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be a valid URL towards PostgreSQL database")
}

fn create_like_string<T: ToString>(search_string: T) -> String {
    let search_string = search_string.to_string();
    let search_string = if !search_string.starts_with("%") {
        format!("%{search_string}")
    } else {
        search_string.clone()
    };

    let search_string = if !search_string.ends_with("%") {
        format!("{search_string}%")
    } else {
        search_string.clone()
    };
    search_string
}
