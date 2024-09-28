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

use crate::model::commands::{
    AddressDetailRegisterSubCommand, FirstOperatorRegisterCommand, MemberDetailRegisterSubCommand,
    MemberRegisterCommand,
};

pub trait AsMemberDetailRegisterSubCommand {
    fn member_detail(&self) -> MemberDetailRegisterSubCommand;
}

pub trait AsAddressDetailRegisterSubCommand {
    fn member_address_detail(&self) -> AddressDetailRegisterSubCommand;
}

pub trait AsAllMemberRegisterSubCommands:
    AsMemberDetailRegisterSubCommand + AsAddressDetailRegisterSubCommand
{
}

impl AsMemberDetailRegisterSubCommand for MemberRegisterCommand {
    fn member_detail(&self) -> MemberDetailRegisterSubCommand {
        MemberDetailRegisterSubCommand {
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            email_address: self.email_address.clone(),
            phone_number: self.phone_number.clone(),
        }
    }
}

impl AsMemberDetailRegisterSubCommand for FirstOperatorRegisterCommand {
    fn member_detail(&self) -> MemberDetailRegisterSubCommand {
        MemberDetailRegisterSubCommand {
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            email_address: self.email_address.clone(),
            phone_number: self.phone_number.clone(),
        }
    }
}

impl AsAddressDetailRegisterSubCommand for MemberRegisterCommand {
    fn member_address_detail(&self) -> AddressDetailRegisterSubCommand {
        AddressDetailRegisterSubCommand {
            street: self.street.clone(),
            house_number: self.house_number.clone(),
            house_number_postfix: self.house_number_postfix.clone(),
            postal_code: self.postal_code.clone(),
            domicile: self.domicile.clone(),
        }
    }
}

impl AsAddressDetailRegisterSubCommand for FirstOperatorRegisterCommand {
    fn member_address_detail(&self) -> AddressDetailRegisterSubCommand {
        AddressDetailRegisterSubCommand {
            street: self.street.clone(),
            house_number: self.house_number.clone(),
            house_number_postfix: self.house_number_postfix.clone(),
            postal_code: self.postal_code.clone(),
            domicile: self.domicile.clone(),
        }
    }
}

impl AsAllMemberRegisterSubCommands for MemberRegisterCommand {}

impl AsAllMemberRegisterSubCommands for FirstOperatorRegisterCommand {}
