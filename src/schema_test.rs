// @generated automatically by Diesel CLI.

diesel::table! {
    member_address_details (id) {
        id -> Nullable<Integer>,
        street -> Text,
        house_number -> Integer,
        house_number_postfix -> Nullable<Text>,
        postal_code -> Text,
        domicile -> Text,
    }
}

diesel::table! {
    member_details (id) {
        id -> Nullable<Integer>,
        first_name -> Text,
        last_name -> Text,
        email_address -> Text,
        phone_number -> Text,
    }
}

diesel::table! {
    member_role_associations (member_id, system_role) {
        member_id -> Integer,
        system_role -> Integer,
    }
}

diesel::table! {
    members (id) {
        id -> Nullable<Integer>,
        member_details_id -> Integer,
        member_address_details_id -> Integer,
        musical_instrument_id -> Nullable<Integer>,
        picture_asset_id -> Nullable<Text>,
        activated -> Bool,
        creation_time -> Timestamp,
        activation_string -> Text,
        activation_time -> Timestamp,
        allow_privacy_info_sharing -> Bool,
        nonce -> Text,
    }
}

diesel::table! {
    musical_instruments (id) {
        id -> Nullable<Integer>,
        name -> Text,
        wikipedia_url -> Nullable<Text>,
    }
}

diesel::table! {
    page_access_policies (page_id, system_role) {
        page_id -> Integer,
        system_role -> Integer,
    }
}

diesel::table! {
    pages (id) {
        id -> Nullable<Integer>,
        content_asset -> Text,
        parent_id -> Nullable<Integer>,
        icon_asset -> Nullable<Text>,
        event_date -> Nullable<Date>,
        etag -> Text,
    }
}

diesel::table! {
    workgroup_member_relationships (workgroup_id, member_id) {
        workgroup_id -> Integer,
        member_id -> Integer,
    }
}

diesel::table! {
    workgroup_role_associations (workgroup_id, system_role) {
        workgroup_id -> Integer,
        system_role -> Integer,
    }
}

diesel::table! {
    workgroups (id) {
        id -> Nullable<Integer>,
        name -> Text,
    }
}

diesel::joinable!(member_role_associations -> members (member_id));
diesel::joinable!(members -> member_address_details (member_address_details_id));
diesel::joinable!(members -> member_details (member_details_id));
diesel::joinable!(members -> musical_instruments (musical_instrument_id));
diesel::joinable!(page_access_policies -> pages (page_id));
diesel::joinable!(workgroup_member_relationships -> members (member_id));
diesel::joinable!(workgroup_member_relationships -> workgroups (workgroup_id));
diesel::joinable!(workgroup_role_associations -> workgroups (workgroup_id));

diesel::allow_tables_to_appear_in_same_query!(
    member_address_details,
    member_details,
    member_role_associations,
    members,
    musical_instruments,
    page_access_policies,
    pages,
    workgroup_member_relationships,
    workgroup_role_associations,
    workgroups,
);
