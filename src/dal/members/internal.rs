use crate::generic::security::generate_encoded_nonce;
use crate::model::members::{Member, MemberAddressDetail, MemberDetail, MemberRegistrationData};
use crate::model::setup::FirstOperator;
use chrono::TimeDelta;
use std::ops::Add;

pub(super) fn create_member_record(
    activation_string: &str,
    mad_id: i32,
    md_id: i32,
    activation_delta: TimeDelta,
) -> Member {
    let data = Member {
        id: 0,
        member_address_details_id: mad_id,
        member_details_id: md_id,
        musical_instrument_id: None,
        picture_asset_id: None,
        allow_privacy_info_sharing: false,
        activated: false,
        activation_string: activation_string.to_string(),
        activation_time: chrono::Utc::now().add(activation_delta).naive_utc(),
        creation_time: chrono::Utc::now().naive_utc(),
        nonce: generate_encoded_nonce(),
    };
    data
}

pub(super) fn member_detail_from_first_operator(operator: &FirstOperator) -> MemberDetail {
    let member_detail = MemberDetail {
        id: 0,
        first_name: operator.first_name.clone(),
        last_name: operator.last_name.clone(),
        email_address: operator.email_address.clone(),
        phone_number: operator.phone_number.clone(),
    };
    member_detail
}

pub(super) fn member_address_detail_from_first_operator(
    operator: &FirstOperator,
) -> MemberAddressDetail {
    let member_address_detail = MemberAddressDetail {
        id: 0,
        street: operator.street.clone(),
        house_number: operator.house_number.clone(),
        house_number_postfix: operator.house_number_postfix.clone(),
        postal_code: operator.postal_code.clone(),
        domicile: operator.domicile.clone(),
    };
    member_address_detail
}

pub(super) fn member_detail_from_member_registration_data(
    registration_data: &MemberRegistrationData,
) -> MemberDetail {
    let member_detail = MemberDetail {
        id: 0,
        first_name: registration_data.first_name.clone(),
        last_name: registration_data.last_name.clone(),
        email_address: registration_data.email_address.clone(),
        phone_number: registration_data.phone_number.clone(),
    };
    member_detail
}

pub(super) fn member_address_detail_from_member_registration_data(
    registration_data: &MemberRegistrationData,
) -> MemberAddressDetail {
    let member_address_detail = MemberAddressDetail {
        id: 0,
        street: registration_data.street.clone(),
        house_number: registration_data.house_number.clone(),
        house_number_postfix: registration_data.house_number_postfix.clone(),
        postal_code: registration_data.postal_code.clone(),
        domicile: registration_data.domicile.clone(),
    };
    member_address_detail
}
