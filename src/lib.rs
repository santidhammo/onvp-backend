use diesel::{r2d2, PgConnection};

pub mod api;
pub mod dal;
pub mod model;
pub mod schema;
pub(crate) mod security;

type DbPool = r2d2::Pool<r2d2::ConnectionManager<PgConnection>>;

pub fn initialize_db_pool() -> DbPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = r2d2::ConnectionManager::<PgConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("database URL should be a valid URL towards PostgreSQL database")
}
