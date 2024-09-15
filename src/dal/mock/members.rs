use crate::dal::members::*;
use crate::dal::DbConnection;
use crate::model::members::{MemberAddressDetail, MemberDetail};
use crate::model::security::Role;
use crate::Result;
use chrono::TimeDelta;
use rand::distributions::{Alphanumeric, DistString};
use rand::{thread_rng, Rng};

pub fn create(
    conn: &mut DbConnection,
    count: i32,
    activation_delta: TimeDelta,
    role: Role,
) -> Result<()> {
    for _ in 0..count {
        let member_detail = MemberDetail {
            id: 0,
            first_name: Alphanumeric.sample_string(&mut thread_rng(), 8),
            last_name: Alphanumeric.sample_string(&mut thread_rng(), 8),
            email_address: format!(
                "{}@{}.{}",
                Alphanumeric.sample_string(&mut thread_rng(), 8),
                Alphanumeric.sample_string(&mut thread_rng(), 8),
                Alphanumeric.sample_string(&mut thread_rng(), 3)
            ),
            phone_number: create_phone_number(),
        };

        let member_address_detail = MemberAddressDetail {
            id: 0,
            street: Alphanumeric.sample_string(&mut thread_rng(), 32),
            house_number: thread_rng().gen_range(1..100),
            house_number_postfix: create_house_number_postfix(),
            postal_code: create_postal_code(),
            domicile: Alphanumeric.sample_string(&mut thread_rng(), 8),
        };

        let activation_string = Alphanumeric.sample_string(&mut thread_rng(), 32);

        create_inactive_member(
            conn,
            member_address_detail,
            member_detail,
            &activation_string,
            activation_delta,
            role,
        )?;
    }

    Ok(())
}

fn create_phone_number() -> String {
    let mut phone_number = "+".to_string();
    for _ in 0..10 {
        let num = thread_rng().gen_range(1..10);
        phone_number.push_str(&num.to_string());
    }
    phone_number
}

fn create_house_number_postfix() -> Option<String> {
    let house_number_postfix = Alphanumeric.sample_string(&mut thread_rng(), 2);
    let house_number_postfix = if house_number_postfix.contains(char::is_alphabetic) {
        Some(house_number_postfix)
    } else {
        None
    };
    house_number_postfix
}

fn create_postal_code() -> String {
    let mut postal_code = String::new();
    for _ in 0..4 {
        let num = thread_rng().gen_range(0..10);
        postal_code.push_str(&num.to_string());
    }
    for _ in 0..2 {
        let num: u8 = thread_rng().gen_range(65..=90);
        let c = num as char;
        postal_code.push(c);
    }

    postal_code
}
