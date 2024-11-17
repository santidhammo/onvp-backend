/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024.  Sjoerd van Leent
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use diesel::r2d2::ConnectionManager;
use diesel::*;

pub type DatabaseConnectionPool = r2d2::Pool<ConnectionManager<DatabaseConnection>>;
pub type DatabaseConnection = PgConnection;

pub fn initialize_database_connection_pool() -> DatabaseConnectionPool {
    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    let manager = ConnectionManager::<DatabaseConnection>::new(conn_spec);
    r2d2::Pool::builder()
        .build(manager)
        .expect("storage URL should be a valid URL towards PostgreSQL storage")
}
