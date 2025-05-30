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

use ed25519_compact::{KeyPair, PublicKey, SecretKey};
use log::info;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub mod config;
pub mod endpoints;
pub mod middleware;
pub mod server;

fn load_key_pair() -> (SecretKey, PublicKey) {
    let keys_location = env::var("JWT_KEYS").expect("JWT_KEYS should be set");
    info!("Loading JWT keys from {}", keys_location);
    let path = Path::new(&keys_location);
    let mut pem = String::new();
    let _ = File::open(path)
        .expect("JWT_KEYS should exist")
        .read_to_string(&mut pem)
        .expect("JWT_KEYS should be readable");
    let KeyPair {
        sk: secret_key,
        pk: public_key,
    } = KeyPair::from_pem(&pem)
        .expect("Key pair should be created with the specified file in JWT_KEYS");
    (secret_key, public_key)
}
