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
use crate::dal::DbConnection;
use crate::generic::result::BackendResult;
use crate::generic::Injectable;
use crate::repositories::traits::MemberPictureRepository;
use crate::schema::members;
use actix_web::web::Data;
use diesel::{ExpressionMethods, RunQueryDsl};
use std::sync::Arc;

pub struct Implementation;

impl MemberPictureRepository for Implementation {
    fn save_by_member_id(
        &self,
        conn: &mut DbConnection,
        member_id: i32,
        picture_asset_id: &str,
    ) -> BackendResult<()> {
        diesel::update(members::table)
            .filter(members::id.eq(member_id))
            .set(members::picture_asset_id.eq(picture_asset_id))
            .execute(conn)?;
        Ok(())
    }
}

impl Injectable<(), dyn MemberPictureRepository> for Implementation {
    fn injectable(_: ()) -> Data<dyn MemberPictureRepository> {
        let arc: Arc<dyn MemberPictureRepository> = Arc::new(Self);
        Data::from(arc)
    }
}
