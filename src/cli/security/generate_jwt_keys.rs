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

use ed25519_compact::KeyPair;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[allow(dead_code)] // Cargo thinks this is dead code, but it is not
fn main() -> Result<(), Box<dyn Error>> {
    let keypair = KeyPair::generate();

    let pem_content = keypair.to_pem();

    let mut pb = PathBuf::new();
    pb.push("keypair.pem");
    let mut file = File::create(pb.clone())?;
    write!(file, "{}", pem_content)?;

    println!("Keys written to: keypair.pem");

    let canonical = fs::canonicalize(&pb)?;
    println!(
        "Include the keys in the environment: JWT_KEYS={}",
        canonical.to_string_lossy()
    );

    Ok(())
}
