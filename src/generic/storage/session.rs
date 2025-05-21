/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2024-2025.  Sjoerd van Leent
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
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::storage::database::{
    DatabaseBackend, DatabaseConnection, DatabaseConnectionPool, DatabaseTransactionBuilder,
};
use crate::generic::Injectable;
use actix_jwt_auth_middleware::FromRequest;
use actix_web::web::Data;
use diesel::backend::Backend;
use diesel::connection::{AnsiTransactionManager, TransactionManager};
use diesel::query_builder::{QueryBuilder, QueryFragment};
use diesel::r2d2::ConnectionManager;
use log::info;
use r2d2::PooledConnection;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, FromRequest)]
pub struct Session {
    first_run: Rc<Mutex<AtomicBool>>,
    conn: Rc<Mutex<Option<PooledConnection<ConnectionManager<DatabaseConnection>>>>>,
}

impl Session {
    pub fn run<F, R>(&mut self, f: F) -> BackendResult<R>
    where
        F: FnOnce(&mut DatabaseConnection) -> BackendResult<R>,
    {
        let conn_lock = self.conn.lock();
        let atomic_first_run_lock = self.first_run.lock();

        if let (Ok(mut conn), Ok(mut atomic_first_run)) = (conn_lock, atomic_first_run_lock) {
            if let Some(conn) = conn.as_mut() {
                let conn: &mut DatabaseConnection = &mut *conn;
                let first_run = atomic_first_run.get_mut();
                if *first_run {
                    let sql = Self::build_start_transaction_query(conn)?;
                    info!("Starting session transaction");
                    AnsiTransactionManager::begin_transaction_sql(conn, &sql)?;
                    *first_run = false;
                }
                f(conn)
            } else {
                Err(BackendError::bad())
            }
        } else {
            Err(BackendError::bad())
        }
    }

    fn build_start_transaction_query(conn: &mut DatabaseConnection) -> BackendResult<String> {
        let mut query_builder = <DatabaseBackend as Backend>::QueryBuilder::default();
        let builder: DatabaseTransactionBuilder = conn.build_transaction().deferrable();
        builder.to_sql(&mut query_builder, &DatabaseBackend {})?;
        let sql = query_builder.finish();
        Ok(sql)
    }

    pub fn rollback(&mut self) -> BackendResult<()> {
        let conn_lock = self.conn.lock();
        let atomic_first_run_lock = self.first_run.lock();
        if let (Ok(mut conn), Ok(atomic_first_run)) = (conn_lock, atomic_first_run_lock) {
            if let Some(mut conn) = conn.take() {
                let first_run = atomic_first_run.load(Ordering::Relaxed);
                if !first_run {
                    let conn: &mut DatabaseConnection = &mut *conn;
                    info!("Rolling back session transaction");
                    AnsiTransactionManager::rollback_transaction(conn)?;
                }
            }
            Ok(())
        } else {
            Err(BackendError::bad())
        }
    }

    pub fn commit(&mut self) -> BackendResult<()> {
        let conn_lock = self.conn.lock();
        let atomic_first_run_lock = self.first_run.lock();
        if let (Ok(mut conn), Ok(atomic_first_run)) = (conn_lock, atomic_first_run_lock) {
            if let Some(mut conn) = conn.take() {
                let first_run = atomic_first_run.load(Ordering::Relaxed);
                if !first_run {
                    let conn: &mut DatabaseConnection = &mut *conn;
                    info!("Committing session transaction");
                    AnsiTransactionManager::commit_transaction(conn)?;
                }
            }
            Ok(())
        } else {
            Err(BackendError::bad())
        }
    }
}
pub trait SessionManager {
    fn prepare(&self) -> BackendResult<Session>;
}

pub struct DefaultSessionManagerImplementation {
    pool: DatabaseConnectionPool,
}

impl SessionManager for DefaultSessionManagerImplementation {
    fn prepare(&self) -> BackendResult<Session> {
        let conn = self.pool.get()?;

        Ok(Session {
            first_run: Rc::new(Mutex::new(AtomicBool::new(true))),
            conn: Rc::new(Mutex::new(Some(conn))),
        })
    }
}

impl Injectable<DatabaseConnectionPool, dyn SessionManager>
    for DefaultSessionManagerImplementation
{
    fn make(pool: &DatabaseConnectionPool) -> Data<dyn SessionManager> {
        let implementation = Self { pool: pool.clone() };
        let arc: Arc<dyn SessionManager> = Arc::new(implementation);
        Data::from(arc)
    }
}

#[cfg(test)]
pub struct FauxSessionManagerImplementation;

#[cfg(test)]
impl SessionManager for FauxSessionManagerImplementation {
    fn prepare(&self) -> BackendResult<Session> {
        Ok(Session {
            first_run: Rc::new(Mutex::new(AtomicBool::new(true))),
            conn: Rc::new(Mutex::new(None)),
        })
    }
}

#[cfg(test)]
impl Injectable<(), dyn SessionManager> for FauxSessionManagerImplementation {
    fn make((): &()) -> Data<dyn SessionManager> {
        let implementation = Self;
        let arc: Arc<dyn SessionManager> = Arc::new(implementation);
        Data::from(arc)
    }
}
