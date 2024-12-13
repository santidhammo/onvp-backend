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
use crate::generic::result::BackendResult;
use crate::generic::storage::session::Session;
use crate::generic::Injectable;
use crate::model::storage::entities::MailTemplate;
use crate::repositories::definitions::MailTemplateRepository;
use crate::schema::mail_templates;
use actix_web::web::Data;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;

pub struct Implementation {}

impl MailTemplateRepository for Implementation {
    fn create(&self, session: &mut Session, mail_template: MailTemplate) -> BackendResult<()> {
        session.run(|conn| {
            let _ = diesel::insert_into(mail_templates::table)
                .values(mail_template)
                .returning(mail_templates::id)
                .execute(conn)?;
            Ok(())
        })
    }

    fn update(&self, session: &mut Session, mail_template: MailTemplate) -> BackendResult<()> {
        session.run(|conn| {
            diesel::update(mail_templates::table)
                .filter(mail_templates::id.eq(mail_template.id))
                .set(mail_template)
                .execute(conn)?;
            Ok(())
        })
    }

    fn delete(&self, session: &mut Session, mail_template_id: i32) -> BackendResult<()> {
        session.run(|conn| {
            diesel::delete(mail_templates::table)
                .filter(mail_templates::id.eq(mail_template_id))
                .execute(conn)?;
            Ok(())
        })
    }

    fn find_by_id(&self, session: &mut Session, image_id: i32) -> BackendResult<MailTemplate> {
        session.run(|conn| {
            let result = mail_templates::table
                .filter(mail_templates::id.eq(image_id))
                .select(MailTemplate::as_select())
                .first::<MailTemplate>(conn)?;
            Ok(result)
        })
    }

    fn list(&self, session: &mut Session) -> BackendResult<Vec<(i32, String)>> {
        session.run(|conn| {
            let result: Vec<(i32, String)> = mail_templates::table
                .select((mail_templates::id, mail_templates::name))
                .load(conn)?;
            Ok(result)
        })
    }
}

impl Injectable<(), dyn MailTemplateRepository> for Implementation {
    fn make(_: &()) -> Data<dyn MailTemplateRepository> {
        let arc: Arc<dyn MailTemplateRepository> = Arc::new(Self {});
        Data::from(arc)
    }
}
