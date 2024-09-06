// @generated automatically by Diesel CLI.

diesel::table! {
    member_address_details (id) {
        id -> Int4,
        street -> Varchar,
        house_number -> Int4,
        house_number_postfix -> Nullable<Varchar>,
        postal_code -> Varchar,
        domicile -> Varchar,
    }
}

diesel::table! {
    member_details (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        email_address -> Varchar,
        phone_number -> Varchar,
    }
}

diesel::table! {
    member_role_associations (member_id, system_role) {
        member_id -> Int4,
        system_role -> Int4,
    }
}

diesel::table! {
    members (id) {
        id -> Int4,
        member_details_id -> Int4,
        member_address_details_id -> Int4,
        musical_instrument_id -> Nullable<Int4>,
        picture_asset_id -> Nullable<Varchar>,
        activated -> Bool,
        creation_time -> Timestamp,
        activation_string -> Varchar,
        activation_time -> Timestamp,
        allow_privacy_info_sharing -> Bool,
    }
}

diesel::table! {
    musical_instruments (id) {
        id -> Int4,
        name -> Varchar,
        wikipedia_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    page_access_policies (page_id, system_role) {
        page_id -> Int4,
        system_role -> Int4,
    }
}

diesel::table! {
    pages (id) {
        id -> Int4,
        content_asset -> Varchar,
        parent_id -> Nullable<Int4>,
        icon_asset -> Nullable<Varchar>,
        event_date -> Nullable<Date>,
        etag -> Varchar,
    }
}

diesel::table! {
    workgroup_member_relationships (workgroup_id, member_id) {
        workgroup_id -> Int4,
        member_id -> Int4,
    }
}

diesel::table! {
    workgroup_role_associations (workgroup_id, system_role) {
        workgroup_id -> Int4,
        system_role -> Int4,
    }
}

diesel::table! {
    workgroups (id) {
        id -> Int4,
        name -> Varchar,
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
