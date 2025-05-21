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
use crate::model::interface::commands::{
    CreateMailTemplateCommand, CreatePageCommand, ImageUploadCommand,
    RegisterMusicalInstrumentCommand, UpdateMailTemplateCommand, UpdateMusicalInstrumentCommand,
    UpdatePageCommand, WorkgroupRegisterCommand, WorkgroupUpdateCommand,
};
use crate::model::interface::sub_commands;
use crate::model::storage::extended_entities::ExtendedMember;
use diesel::{AsChangeset, Insertable, Queryable, Selectable};

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::members)]
pub struct Member {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub member_details_id: i32,
    pub member_address_details_id: i32,
    pub musical_instrument_id: Option<i32>,
    pub picture_asset_id: Option<String>,
    pub activated: bool,
    pub creation_time: chrono::NaiveDateTime,
    pub activation_string: String,
    pub activation_time: chrono::NaiveDateTime,
    pub allow_privacy_info_sharing: bool,
    pub nonce: String,
    pub description: Option<String>,
}

impl From<&ExtendedMember> for Member {
    fn from(value: &ExtendedMember) -> Self {
        Self {
            id: value.id,
            musical_instrument_id: value.musical_instrument_id,
            picture_asset_id: value.picture_asset_id.clone(),
            activated: value.activated,
            creation_time: value.creation_time,
            activation_string: value.activation_string.clone(),
            activation_time: value.activation_time,
            allow_privacy_info_sharing: value.allow_privacy_info_sharing,
            nonce: value.nonce.clone(),
            member_details_id: value.member_detail.id,
            member_address_details_id: value.member_address_detail.id,
            description: value.description.clone(),
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::member_details)]
pub struct MemberDetail {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email_address: String,
    pub phone_number: String,
}

impl MemberDetail {
    pub(crate) fn name(&self) -> String {
        let mut s = String::new();
        s.push_str(&self.first_name);
        s.push_str(" ");
        s.push_str(&self.last_name);
        s
    }
}

impl From<&sub_commands::DetailRegisterSubCommand> for MemberDetail {
    fn from(input: &sub_commands::DetailRegisterSubCommand) -> Self {
        Self {
            id: 0, // Skipped during creation
            first_name: input.first_name.clone(),
            last_name: input.last_name.clone(),
            email_address: input.email_address.clone(),
            phone_number: input.phone_number.clone(),
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::member_address_details)]
pub struct MemberAddressDetail {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub street: String,
    pub house_number: i32,
    pub house_number_postfix: Option<String>,
    pub postal_code: String,
    pub domicile: String,
}

impl MemberAddressDetail {
    pub(crate) fn gdpr_fake() -> MemberAddressDetail {
        Self {
            id: 0, // Skipped during creation
            street: "".to_owned(),
            house_number: 0,
            house_number_postfix: None,
            postal_code: "0000AA".to_owned(),
            domicile: "".to_owned(),
        }
    }
}

impl From<&sub_commands::AddressRegisterSubCommand> for MemberAddressDetail {
    fn from(input: &sub_commands::AddressRegisterSubCommand) -> Self {
        Self {
            id: 0, // Skipped during creation
            street: input.street.clone(),
            house_number: input.house_number.clone(),
            house_number_postfix: input.house_number_postfix.clone(),
            postal_code: input.postal_code.clone(),
            domicile: input.domicile.clone(),
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::workgroups)]
pub struct Workgroup {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub name: String,
}

impl From<&WorkgroupRegisterCommand> for Workgroup {
    fn from(input: &WorkgroupRegisterCommand) -> Self {
        Self {
            id: 0, // Skipped during creation

            name: input.name.to_string(),
        }
    }
}

/// Merges the update command into an existing extended member
impl From<(&Workgroup, &WorkgroupUpdateCommand)> for Workgroup {
    fn from((origin, command): (&Workgroup, &WorkgroupUpdateCommand)) -> Self {
        let mut cloned = origin.clone();
        cloned.name = command.name.clone();
        cloned
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::pages)]
pub struct Page {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub content_asset: String,
    pub parent_id: Option<i32>,
    pub icon_asset: Option<String>,
    pub event_date: Option<chrono::NaiveDate>,
    pub etag: String,
    pub title: String,
    pub order_number: i32,
    pub end_event_date: Option<chrono::NaiveDate>,
}

impl From<&CreatePageCommand> for Page {
    fn from(value: &CreatePageCommand) -> Self {
        Self {
            id: 0, // Skipped during creation

            content_asset: crate::generate_asset_id(),
            parent_id: None,
            icon_asset: None,
            event_date: value
                .event_date
                .clone()
                .map(|d| d.as_validated().ok())
                .flatten(),
            etag: crate::generate_asset_id(),
            title: value.title.clone(),
            order_number: 0,
            end_event_date: value
                .event_date
                .clone()
                .map(|d| d.as_validated().ok())
                .flatten(),
        }
    }
}

impl From<(&Page, &UpdatePageCommand)> for Page {
    fn from((origin, command): (&Page, &UpdatePageCommand)) -> Self {
        let mut cloned = origin.clone();
        cloned.event_date = command
            .event_date
            .clone()
            .map(|d| d.as_validated().ok())
            .flatten();
        cloned.title = command.title.clone();
        cloned
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::images)]
pub struct Image {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub title: String,
    pub asset: String,
}

impl From<&ImageUploadCommand> for Image {
    fn from(value: &ImageUploadCommand) -> Self {
        Self {
            id: 0, // Skipped during creation

            title: value.title.clone(),
            asset: crate::generate_asset_id(),
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::musical_instruments)]
pub struct MusicalInstrument {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub name: String,
    pub wikipedia_url: Option<String>,
}

impl From<&RegisterMusicalInstrumentCommand> for MusicalInstrument {
    fn from(value: &RegisterMusicalInstrumentCommand) -> Self {
        Self {
            id: 0, // Skipped during creation

            name: value.name.clone(),
            wikipedia_url: value.wikipedia_url.clone(),
        }
    }
}

impl From<(&MusicalInstrument, &UpdateMusicalInstrumentCommand)> for MusicalInstrument {
    fn from((origin, command): (&MusicalInstrument, &UpdateMusicalInstrumentCommand)) -> Self {
        Self {
            id: origin.id,
            name: command.name.clone(),
            wikipedia_url: command.wikipedia_url.clone(),
        }
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::mail_templates)]
pub struct MailTemplate {
    #[diesel(skip_insertion)]
    pub id: i32,
    pub name: String,
    pub body: String,
}

impl From<&CreateMailTemplateCommand> for MailTemplate {
    fn from(command: &CreateMailTemplateCommand) -> Self {
        Self {
            id: 0, // Skipped during creation

            name: command.name.clone(),
            body: command.body.clone(),
        }
    }
}

impl From<(&MailTemplate, &UpdateMailTemplateCommand)> for MailTemplate {
    fn from((origin, command): (&MailTemplate, &UpdateMailTemplateCommand)) -> Self {
        Self {
            id: origin.id,
            name: origin.name.clone(),
            body: command.body.clone(),
        }
    }
}
