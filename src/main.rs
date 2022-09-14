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

// def get_public_key_comment():
//     return ' devkit-client:{}@{}'.format(
//         getpass.getuser(),
//         socket.gethostname(),
//     )
//
//
// def get_public_key(key):
//     public_key = (
//         'ssh-rsa ' + key.get_base64() + get_public_key_comment() + '\n')
//     return public_key

fn get_public_key_comment() -> String {
    let user = "nanoteck137";
    let hostname = "testing";
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

    println!("{}", get_public_key(&key));

    let mut file = File::create("decker_devkit_key").ok()?;
    file.write(&key.private_key_to_pem().ok()?).ok()?;

    Some(())
}

fn get_devkit_key() -> Option<Rsa<Private>> {
    None
}

fn main() {
    setup().expect("Failed to setup device");

    // TODO(patrik): Generate RSA Key for SSH

    // let key = openssl::rsa::Rsa::generate(2048).unwrap();
    // let pub_key = key.public_key_to_der().unwrap();
    // let pub_key = base64::encode(pub_key);
    // println!("Key: {}", pub_key);
    //
    // println!(
    //     "Key: {}",
    //     std::str::from_utf8(&key.public_key_to_pem().unwrap()).unwrap()
    // );
    //
    // let private_key = &key.private_key_to_pem().unwrap();
    // let pem_key = std::str::from_utf8(private_key).unwrap();
    // println!("Key: {}", pem_key);
    //
    // let priv_key = openssl::rsa::Rsa::private_key_from_pem("Hello World".as_bytes()).unwrap();
    //
    // let private_key = &priv_key.private_key_to_pem().unwrap();
    // let pem_key = std::str::from_utf8(private_key).unwrap();
    // println!(
    //     "Key 2: {}",
    //     std::str::from_utf8(&priv_key.public_key_to_pem().unwrap()).unwrap()
    // );
    // println!("Key 2: {}", pem_key);

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
