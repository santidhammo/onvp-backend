use ed25519_compact::KeyPair;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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
