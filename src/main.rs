// NOTE(patrik):
// Offical Valve devkit client: https://gitlab.steamos.cloud/devkit/steamos-devkit
// URL: http://machine-ip:32000
// Magic: 900b919520e4cf601998a71eec318fec
//   - From: https://gitlab.steamos.cloud/devkit/steamos-devkit/-/blob/main/client/devkit_client/__init__.py
//   - NOTE: Maybe not needed
// Endpoints
//   - /properties.json
//   - /login-name
//   - /register
//     - Data: SSH_KEY + " " + MAGIC

// TODO(patrik):
//  Deployment = game
//
//  - Setup the device
//    - Copy over a helper program
//  - Helper Program
//    - Create new depolyment
//    - Delete depolyment
//  - Setup the device for the deployment
//    - Create the directory structure
//    - Copy over the files
//  - Create a Steam shortcut
//

use serde_json::Value;

use clap::Parser;

use std::fs::File;
use std::io::{Write, Read};
use std::process::Command;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Error {
    RegisterRequestFailed(reqwest::Error),
    FailedToRetriveRegisterRequestText(reqwest::Error),
    FailedToParseErrorJson(serde_json::Error),
    FailedToRegister(String),
    FailedToRegisterWithoutMessage,
    RegisterRequestUnknownStatus(u16),

    FailedToOpenPublicKeyFile(std::io::Error),
    FailedToReadPublicKeyFile(std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Parser, Debug)]
struct Args {
    command: String,
}

fn get_public_key() -> Result<String> {
    let mut file = File::open("decker_devkit_key.pub")
        .map_err(|e| Error::FailedToOpenPublicKeyFile(e))?;

    let mut result = String::new();
    file.read_to_string(&mut result)
        .map_err(|e| Error::FailedToReadPublicKeyFile(e))?;

    Ok(result)
}

fn register(addr: &str) -> Result<()> {
    // TODO(patrik): Move port
    let url = format!("http://{}:32000/register", addr);

    let mut public_key = get_public_key()?;
    // TODO(patrik): We might not need to have the magic value because
    // registering without it works

    // Remove the newline
    public_key.pop();

    // Push the magic phrase
    public_key.push_str(" 900b919520e4cf601998a71eec318fec\n");

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(url)
        .body(public_key)
        .send()
        .map_err(|e| Error::RegisterRequestFailed(e))?;

    if res.status().is_client_error() {
        let res = res
            .text()
            .map_err(|e| Error::FailedToRetriveRegisterRequestText(e))?;
        let res: Value = serde_json::from_str(&res)
            .map_err(|e| Error::FailedToParseErrorJson(e))?;
        if let Some(error) = res.get("error") {
            if let Some(error) = error.as_str() {
                Err(Error::FailedToRegister(error.to_owned()))
            } else {
                Err(Error::FailedToRegisterWithoutMessage)
            }
        } else {
            Err(Error::FailedToRegisterWithoutMessage)
        }
    } else if res.status().is_success() {
        Ok(())
    } else {
        Err(Error::RegisterRequestUnknownStatus(res.status().as_u16()))
    }
}

fn execute_simple_ssh(addr: &str, cmd: &str) -> std::process::Output {
    let username = "deck";
    let host = format!("{}@{}", username, addr);

    Command::new("ssh")
        .arg("-oBatchMode=yes")
        .arg("-i")
        .arg("decker_devkit_key")
        .arg(host)
        .arg(cmd)
        .output()
        .expect("Failed to execute ssh")
}

fn execute_simple_scp<S, D>(
    addr: &str,
    source: S,
    dest: D,
) -> std::process::Output
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    let username = "deck";
    let host = format!("{}@{}", username, addr);

    let source = source.as_ref();
    let dest = dest.as_ref();
    let dest = format!("{}:{}", host, dest.to_str().unwrap());

    Command::new("scp")
        .arg("-oBatchMode=yes")
        .arg("-i")
        .arg("decker_devkit_key")
        .arg(source)
        .arg(dest)
        .output()
        .expect("Failed to execute scp")
}

fn execute_simple_rsync<S, D>(
    addr: &str,
    source: S,
    dest: D,
) -> std::process::Output
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    let username = "deck";
    let host = format!("{}@{}", username, addr);

    let source = source.as_ref();
    let dest = dest.as_ref();
    let dest = format!("{}:{}", host, dest.to_str().unwrap());

    Command::new("rsync")
        .arg("-e")
        .arg("ssh -i decker_devkit_key")
        .arg("-r")
        .arg(source)
        .arg(dest)
        .output()
        .expect("Failed to execute rsync")
}

fn check_if_registered(addr: &str) -> bool {
    let output = execute_simple_ssh(addr, "ls");

    output.status.success()
}

fn simple_print_output(output: &std::process::Output) {
    if output.status.success() {
        println!("{}", std::str::from_utf8(&output.stdout).unwrap());
    } else {
        println!("Error: {}", std::str::from_utf8(&output.stderr).unwrap());
    }
}

fn read_file_binary<P>(filepath: P) -> Vec<u8>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filepath).unwrap();

    let mut result = Vec::new();
    file.read_to_end(&mut result).unwrap();

    result
}

fn write_file_binary<P>(filepath: P, data: &[u8])
where
    P: AsRef<Path>,
{
    let mut file = File::create(filepath).unwrap();
    file.write(data).unwrap();
}

fn main() {
    let addr = if let Ok(addr) = std::env::var("DEVKIT_ADDR") {
        addr
    } else {
        panic!("DEVKIT_ADDR not set");
    };

    println!("Device Address: {}", addr);

    if !check_if_registered(&addr) {
        register(&addr).expect("Failed to register device");
    }

    let game_id = "test";
    let exec = "/home/deck/decker-games/test/linux.sh";
    let starting_dir = "/home/deck/decker-games/test/";

    execute_simple_ssh(&addr, "mkdir -p ~/decker");

    let mut exe_path = std::env::current_exe().unwrap();
    exe_path.set_file_name("decker_util");
    execute_simple_scp(&addr, exe_path, "~/decker/decker_util");

    let cmd = format!("~/decker/decker_util prepare-upload {} true", game_id);
    let _output = execute_simple_ssh(&addr, &cmd);
    // simple_print_output(&output);

    let cmd = format!(
        "~/decker/decker_util create-shortcut {} {} {}",
        game_id, exec, starting_dir
    );

    let output = execute_simple_ssh(&addr, &cmd);
    simple_print_output(&output);

    let source = Path::new("../Testing/export/");
    let dest = format!("~/decker-games/{}", game_id);

    let output = execute_simple_rsync(&addr, source, dest);
    simple_print_output(&output);
}
