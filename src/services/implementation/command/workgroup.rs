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
use crate::generic::result::{BackendError, BackendResult};
use crate::generic::storage::database::DatabaseConnectionPool;
use crate::generic::Injectable;
use crate::model::interface::commands::{WorkgroupRegisterCommand, WorkgroupUpdateCommand};
use crate::model::storage::entities::Workgroup;
use crate::repositories::definitions::WorkgroupRepository;
use crate::services::definitions::command::WorkgroupCommandService;
use actix_web::web::Data;
use diesel::Connection;
use std::sync::Arc;

pub struct Implementation {
    pool: DatabaseConnectionPool,
    workgroup_repository: Data<dyn WorkgroupRepository>,
}

impl WorkgroupCommandService for Implementation {
    fn register(&self, command: &WorkgroupRegisterCommand) -> BackendResult<i32> {
        let mut conn = self.pool.get()?;
        self.workgroup_repository
            .register(&mut conn, Workgroup::from(command))
    }

    fn update(&self, workgroup_id: i32, command: &WorkgroupUpdateCommand) -> BackendResult<()> {
        let mut conn = self.pool.get()?;
        conn.transaction::<_, BackendError, _>(|conn| {
            let origin = self.workgroup_repository.find_by_id(conn, workgroup_id)?;
            let new = Workgroup::from((&origin, command));
            self.workgroup_repository.save(conn, new)?;
            Ok(())
        })
    }
}

impl
    Injectable<
        (&DatabaseConnectionPool, &Data<dyn WorkgroupRepository>),
        dyn WorkgroupCommandService,
    > for Implementation
{
    fn injectable(
        (pool, workgroup_repository): (&DatabaseConnectionPool, &Data<dyn WorkgroupRepository>),
    ) -> Data<dyn WorkgroupCommandService> {
        let implementation = Self {
            pool: pool.clone(),
            workgroup_repository: workgroup_repository.clone(),
        };
        let arc: Arc<dyn WorkgroupCommandService> = Arc::new(implementation);
        Data::from(arc)
    }
}
