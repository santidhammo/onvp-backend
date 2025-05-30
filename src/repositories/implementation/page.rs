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
use crate::generic::lazy::SEARCH_PAGE_SIZE;
use crate::generic::result::BackendResult;
use crate::generic::security::ClaimRoles;
use crate::generic::storage::session::Session;
use crate::generic::{search_helpers, Injectable};
use crate::model::primitives::Role;
use crate::model::storage::entities::Page;
use crate::repositories::definitions::PageRepository;
use crate::schema::*;
use actix_web::web::Data;
use chrono::NaiveDate;
use diesel::debug_query;
use diesel::dsl::exists;
use diesel::pg::Pg;
use diesel::prelude::*;
use log::info;
use std::sync::Arc;

pub struct Implementation {
    page_size: usize,
}

impl PageRepository for Implementation {
    fn create(&self, session: &mut Session, page: Page) -> BackendResult<()> {
        let page_id = session.run(|conn| {
            let page_id: i32 = diesel::insert_into(pages::table)
                .values(page)
                .returning(pages::id)
                .get_result(conn)?;
            Ok(page_id)
        })?;

        self.reset_roles(session, page_id)
    }

    fn update(&self, session: &mut Session, page: Page) -> BackendResult<()> {
        session.run(|conn| {
            diesel::update(pages::table)
                .filter(pages::id.eq(page.id))
                .set(page)
                .execute(conn)?;
            Ok(())
        })
    }

    fn set_order_by_id(
        &self,
        session: &mut Session,
        page_id: i32,
        order_number: i32,
    ) -> BackendResult<()> {
        session.run(|conn| {
            diesel::update(pages::table)
                .filter(pages::id.eq(page_id))
                .set((pages::order_number.eq(order_number),))
                .execute(conn)?;
            Ok(())
        })
    }

    fn set_or_unset_parent_id_by_id(
        &self,
        session: &mut Session,
        page_id: i32,
        maybe_parent_id: Option<i32>,
    ) -> BackendResult<()> {
        session.run(|conn| {
            diesel::update(pages::table)
                .filter(pages::id.eq(page_id))
                .set((pages::parent_id.eq(maybe_parent_id),))
                .execute(conn)?;
            Ok(())
        })
    }

    fn find_by_id(&self, session: &mut Session, page_id: i32) -> BackendResult<Page> {
        session.run(|conn| {
            let page = pages::table
                .filter(pages::id.eq(page_id))
                .select(Page::as_select())
                .first::<Page>(conn)?;
            Ok(page)
        })
    }

    fn list_by_parent_id(
        &self,
        session: &mut Session,
        parent_id: i32,
        roles: &ClaimRoles,
    ) -> BackendResult<Vec<Page>> {
        session.run(|conn| {
            let sub_table = page_access_policies::table
                .select(page_access_policies::page_id)
                .distinct()
                .filter(
                    roles
                        .generate_policy_expression(&page_access_policies::system_role)
                        .and(page_access_policies::page_id.eq(pages::id)),
                );

            let pages = if parent_id == 0 {
                let q = pages::table
                    .filter(
                        pages::parent_id
                            .eq(parent_id)
                            .or(pages::parent_id.is_null())
                            .and(exists(sub_table)),
                    )
                    .select(Page::as_select())
                    .order_by(pages::order_number);
                info!("{}", debug_query::<Pg, _>(&q).to_string());
                q.load(conn)?
            } else {
                let q = pages::table
                    .filter(pages::parent_id.eq(parent_id).and(exists(sub_table)))
                    .select(Page::as_select())
                    .order_by(pages::order_number);
                info!("{}", debug_query::<Pg, _>(&q).to_string());
                q.load(conn)?
            };

            Ok(pages)
        })
    }

    fn find_associated_roles_by_id(
        &self,
        session: &mut Session,
        page_id: i32,
    ) -> BackendResult<Vec<Role>> {
        session.run(|conn| {
            let associated_roles: Vec<Role> = page_access_policies::table
                .filter(page_access_policies::page_id.eq(page_id))
                .select(page_access_policies::system_role)
                .load(conn)?;

            Ok(associated_roles)
        })
    }

    fn delete(&self, session: &mut Session, page_id: i32) -> BackendResult<()> {
        session.run(|conn| {
            diesel::delete(pages::table)
                .filter(pages::id.eq(page_id))
                .execute(conn)?;
            Ok(())
        })
    }

    fn reset_roles(&self, session: &mut Session, page_id: i32) -> BackendResult<()> {
        session.run(|conn| {
            diesel::delete(page_access_policies::table)
                .filter(page_access_policies::page_id.eq(page_id))
                .execute(conn)?;

            diesel::insert_into(page_access_policies::table)
                .values((
                    page_access_policies::page_id.eq(page_id),
                    page_access_policies::system_role.eq(Role::Operator),
                ))
                .execute(conn)?;

            Ok(())
        })
    }

    fn assign_roles(
        &self,
        session: &mut Session,
        page_id: i32,
        roles: &Vec<Role>,
    ) -> BackendResult<()> {
        session.run(|conn| {
            for role in roles {
                if role != &Role::Operator {
                    diesel::insert_into(page_access_policies::table)
                        .values((
                            page_access_policies::page_id.eq(page_id),
                            page_access_policies::system_role.eq(role),
                        ))
                        .execute(conn)?;
                }
            }
            Ok(())
        })
    }

    fn search(
        &self,
        session: &mut Session,
        page_offset: usize,
        term: &str,
        roles: &ClaimRoles,
    ) -> BackendResult<(usize, usize, Vec<Page>)> {
        let like_search_string = search_helpers::create_like_string(term);
        let (total_count, pages) = session.run(|conn| {
            let sub_table =
                QueryDsl::select(page_access_policies::table, page_access_policies::page_id)
                    .distinct()
                    .filter(
                        roles
                            .generate_policy_expression(&page_access_policies::system_role)
                            .and(page_access_policies::page_id.eq(pages::id)),
                    );

            let where_expression = pages::title
                .ilike(like_search_string)
                .and(exists(sub_table));

            let total_count: usize = pages::table
                .filter(&where_expression)
                .count()
                .get_result::<i64>(conn)? as usize;

            let result: Vec<(Page,)> = QueryDsl::select(
                QueryDsl::limit(
                    pages::table
                        .filter(&where_expression)
                        .order_by(pages::order_number),
                    self.page_size as i64,
                )
                .offset((page_offset * self.page_size) as i64),
                (Page::as_select(),),
            )
            .load(conn)?;

            Ok((total_count, result.iter().map(|(p,)| p.clone()).collect()))
        })?;
        Ok((total_count, self.page_size, pages))
    }

    fn find_events(
        &self,
        session: &mut Session,
        roles: &ClaimRoles,
        start_date: &NaiveDate,
        end_date: &NaiveDate,
    ) -> BackendResult<Vec<Page>> {
        let mut pages = session.run(|conn| {
            let sub_table =
                QueryDsl::select(page_access_policies::table, page_access_policies::page_id)
                    .distinct()
                    .filter(
                        roles
                            .generate_policy_expression(&page_access_policies::system_role)
                            .and(page_access_policies::page_id.eq(pages::id)),
                    );

            let event_date_filter = pages::event_date.ge(start_date).or(pages::event_date
                .lt(start_date)
                .and(pages::end_event_date.ge(start_date)));

            let where_expression = event_date_filter
                .and(pages::event_date.le(end_date))
                .and(exists(sub_table));

            let result = debug_query::<Pg, _>(&where_expression);
            info!("{}", result.to_string());

            Ok(pages::table
                .filter(&where_expression)
                .order_by(pages::event_date)
                .load::<Page>(conn)?)
        });

        // If there are pages with no end event date, set the end event date to the event date
        if let Ok(pages) = pages.as_mut() {
            for page in &mut pages.iter_mut() {
                if let Some(event_date) = &page.event_date {
                    if page.end_event_date.is_none() {
                        page.end_event_date = Some(event_date.clone())
                    }
                }
            }
        }

        pages
    }
}

impl Injectable<(), dyn PageRepository> for Implementation {
    fn make(_: &()) -> Data<dyn PageRepository> {
        let arc: Arc<dyn PageRepository> = Arc::new(Self {
            page_size: *SEARCH_PAGE_SIZE,
        });
        Data::from(arc)
    }
}
