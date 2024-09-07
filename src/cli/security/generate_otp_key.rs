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
