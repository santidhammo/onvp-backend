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

use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use dotenv::dotenv;
use onvp_backend::generic::storage::database;
use std::env::set_var;
use std::path::PathBuf;

pub(crate) fn setup() -> database::DatabaseConnectionPool {
    dotenv().ok();
    // Correct the storage URL to be using an in-memory, Sqlite storage instead of the configured
    // storage.
    set_var("DATABASE_URL", ":memory:");
    let pool = database::initialize_database_connection_pool();

    let mut conn = pool.get().unwrap();
    run_migrations(&mut conn, setup_migrations());

    pool
}

fn run_migrations(conn: &mut database::DatabaseConnection, fb_migrations: FileBasedMigrations) {
    match conn {
        database::DatabaseConnection::SQLite(sqlite_conn) => {
            sqlite_conn.run_pending_migrations(fb_migrations).unwrap();
        }
        _ => {
            panic!("No PostgreSQL connection expected");
        }
    }
}

fn setup_migrations() -> FileBasedMigrations {
    let mut pb = PathBuf::new();
    pb.push("migrations/sqlite");
    let canon_path = pb.canonicalize().unwrap();
    FileBasedMigrations::from_path(canon_path).unwrap()
}
