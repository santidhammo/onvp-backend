use crate::Error;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use diesel::SqliteConnection;

pub mod members;

pub mod mock;

pub fn connect(pool: &DbPool) -> Result<PooledConnection<ConnectionManager<DbConnection>>, Error> {
    pool.get().map_err(|e| crate::Error::from(e))
}

pub type DbPool = r2d2::Pool<ConnectionManager<DbConnection>>;

#[derive(diesel::MultiConnection)]
pub enum DbConnection {
    PostgreSQL(PgConnection),
    SQLite(SqliteConnection),
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

/// The total count of pages is the total_count divided by page_size, and if the
/// rest is > 0, one more.
fn calculate_page_count(page_size: usize, total_count: usize) -> usize {
    let page_count = (total_count / page_size) + if (total_count % page_size) != 0 { 1 } else { 0 };
    page_count
}
