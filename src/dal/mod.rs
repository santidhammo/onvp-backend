use crate::DbPool;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;

pub mod members;

pub fn get_connection(
    pool: &DbPool,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, String> {
    Ok(pool.get().map_err(|e| e.to_string())?)
}
