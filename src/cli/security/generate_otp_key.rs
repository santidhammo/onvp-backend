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

mod generate_jwt_keys;

use aes_gcm::aead::OsRng;
use aes_gcm::{Aes256Gcm, KeyInit};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let key = Aes256Gcm::generate_key(OsRng);
    let key: [u8; 32] = key.try_into()?;
    use base64::{engine::general_purpose, Engine as _};
    println!(
        "Please add the following information to the environment: OTP_KEY={}",
        general_purpose::STANDARD.encode(&key)
    );
    Ok(())
}
