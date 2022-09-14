// NOTE(patrik):
// Offical Valve devkit client: https://gitlab.steamos.cloud/devkit/steamos-devkit
// URL: http://machine-ip:32000
// Magic: 900b919520e4cf601998a71eec318fec
//   - From: https://gitlab.steamos.cloud/devkit/steamos-devkit/-/blob/main/client/devkit_client/__init__.py
// Endpoints
//   - /properties.json
//   - /login-name
//   - /register
//     - Data: SSH_KEY + " " + MAGIC

use serde_json::Value;

use openssl::pkey::Private;
use openssl::rsa::Rsa;

use std::fs::File;
use std::io::Write;

fn get_public_key_comment() -> String {
    let user = whoami::username();
    let hostname = whoami::hostname();
    format!("devkit-client:{}@{}", user, hostname)
}

fn get_public_key(key: &Rsa<Private>) -> String {
    let pub_key = key.public_key_to_der().unwrap();
    let pub_key = base64::encode(pub_key);

    let comment = get_public_key_comment();
    format!("ssh-rsa {} {}\n", pub_key, comment)
}

fn setup() -> Option<()> {
    let key = openssl::rsa::Rsa::generate(2048).ok()?;

    let private_key_path = "decker_devkit_key";
    println!("Writing the private key to '{}'", private_key_path);
    let mut file = File::create(private_key_path).ok()?;
    file.write(&key.private_key_to_pem().ok()?).ok()?;

    let public_key_path = "decker_devkit_key.pub";
    println!("Writing the public key to '{}'", public_key_path);
    let mut file = File::create("decker_devkit_key.pub").ok()?;
    file.write(get_public_key(&key).as_bytes()).ok()?;

    Some(())
}

fn get_devkit_key() -> Option<Rsa<Private>> {
    None
}

fn main() {
    setup()
        .expect("Failed to setup device");

    let addr = if let Ok(addr) = std::env::var("DEVKIT_ADDR") {
        addr
    } else {
        panic!("DEVKIT_ADDR not set");
    };

    let url = format!("http://{}/properties.json", addr);
    let res = reqwest::blocking::get(url).unwrap().text().unwrap();

    let res: Value = serde_json::from_str(&res).unwrap();
    let has_error = res.get("error").is_some();

    println!("{} {}", res, has_error);
}
