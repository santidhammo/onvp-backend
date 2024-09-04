//! This file contains auxiliary model definitions

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod security {
    use crate::security::Role;
    use diesel::Insertable;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct MemberRoleAssociation {
        #[schema(example = 1)]
        member_id: i32,
        #[schema(example = 1)]
        system_role: i32,
    }
}
pub mod members {
    use diesel::{Identifiable, Insertable, Queryable};
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(
        Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable,
    )]
    #[serde(rename_all = "camelCase")]
    #[diesel(table_name = crate::schema::members)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct Member {
        pub id: Option<i32>,

        #[schema(example = 1)]
        pub member_address_details_id: i32,

        #[schema(example = 1)]
        pub member_details_id: i32,

        #[schema(example = 1)]
        pub musical_instrument_id: Option<i32>,

        #[schema(example = "xyz.png")]
        pub picture_asset_id: Option<String>,

        #[schema(example = false)]
        pub allow_privacy_info_sharing: bool,
    }

    #[derive(
        Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable,
    )]
    #[serde(rename_all = "camelCase")]
    #[diesel(table_name = crate::schema::member_details)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct MemberDetails {
        pub id: Option<i32>,

        #[schema(example = "john_doe")]
        pub user_name: String,

        #[schema(example = "John")]
        pub first_name: String,

        #[schema(example = "Doe")]
        pub last_name: String,

        #[schema(example = "john@doe.void")]
        pub email_address: String,

        #[schema(example = "+99999999999")]
        pub phone_number: String,
    }

    #[derive(
        Serialize, Deserialize, ToSchema, Clone, Debug, Insertable, Queryable, Identifiable,
    )]
    #[serde(rename_all = "camelCase")]
    #[diesel(table_name = crate::schema::member_address_details)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct MemberAddressDetails {
        pub id: Option<i32>,

        #[schema(example = "Orchestra Street")]
        pub street: String,

        #[schema(example = 1)]
        pub house_number: i32,

        #[schema(example = "a")]
        pub house_number_postfix: Option<String>,

        #[schema(example = "9999ZZ")]
        pub postal_code: String,

        #[schema(example = "Tubaton")]
        pub domicile: String,
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FirstOperator {
    #[schema(example = "John")]
    pub first_name: String,

    #[schema(example = "Doe")]
    pub last_name: String,

    #[schema(example = "john@doe.void")]
    pub email_address: String,

    #[schema(example = "john_doe")]
    pub user_name: String,

    #[schema(example = "+99999999999")]
    pub phone_number: String,

    #[schema(example = "Orchestra Road")]
    pub street: String,

    #[schema(example = 1)]
    pub house_number: i32,

    #[schema(example = "a")]
    pub house_number_postfix: Option<String>,

    #[schema(example = "9999ZZ")]
    pub postal_code: String,

    #[schema(example = "Tubaton")]
    pub domicile: String,
}
