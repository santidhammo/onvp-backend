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
use crate::schema::{member_details, pages, workgroups};
use diesel::dsl::{ILike, Like, Or};
use diesel::internal::derives::as_expression::Bound;
use diesel::sql_types::Text;
use diesel::BoolExpressionMethods;

pub struct WorkgroupSearchExpressionGenerator;

impl WorkgroupSearchExpressionGenerator {
    pub fn postgresql(term: &str) -> ILike<workgroups::name, Bound<Text, &str>> {
        use diesel::PgTextExpressionMethods;
        workgroups::name.ilike(term)
    }
    pub fn sqlite(term: &str) -> Like<workgroups::name, Bound<Text, &str>> {
        use diesel::TextExpressionMethods;
        workgroups::name.like(term)
    }
}

pub struct MemberSearchExpressionGenerator;

impl MemberSearchExpressionGenerator {
    pub fn postgresql(
        term: &str,
    ) -> Or<
        Or<
            ILike<member_details::first_name, Bound<Text, &str>>,
            ILike<member_details::last_name, Bound<Text, &str>>,
        >,
        ILike<member_details::email_address, Bound<Text, &str>>,
    > {
        use diesel::PgTextExpressionMethods;
        member_details::first_name
            .ilike(term)
            .or(member_details::last_name.ilike(term))
            .or(member_details::email_address.ilike(term))
    }

    pub fn sqlite(
        term: &str,
    ) -> Or<
        Or<
            Like<member_details::first_name, Bound<Text, &str>>,
            Like<member_details::last_name, Bound<Text, &str>>,
        >,
        Like<member_details::email_address, Bound<Text, &str>>,
    > {
        use diesel::TextExpressionMethods;
        member_details::first_name
            .like(term)
            .or(member_details::last_name.like(term))
            .or(member_details::email_address.like(term))
    }
}

pub struct FacebookSearchExpressionGenerator;

impl FacebookSearchExpressionGenerator {
    pub fn postgresql(
        term: &str,
    ) -> Or<
        ILike<member_details::first_name, Bound<Text, &str>>,
        ILike<member_details::last_name, Bound<Text, &str>>,
    > {
        use diesel::PgTextExpressionMethods;
        member_details::first_name
            .ilike(term)
            .or(member_details::last_name.ilike(term))
    }

    pub fn sqlite(
        term: &str,
    ) -> Or<
        Like<member_details::first_name, Bound<Text, &str>>,
        Like<member_details::last_name, Bound<Text, &str>>,
    > {
        use diesel::TextExpressionMethods;
        member_details::first_name
            .like(term)
            .or(member_details::last_name.like(term))
    }
}

pub struct PageSearchExpressionGenerator;

impl PageSearchExpressionGenerator {
    pub fn postgresql(term: &str) -> ILike<pages::title, Bound<Text, &str>> {
        use diesel::PgTextExpressionMethods;
        pages::title.ilike(term)
    }

    pub fn sqlite(term: &str) -> Like<pages::title, Bound<Text, &str>> {
        use diesel::TextExpressionMethods;
        pages::title.like(term)
    }
}
