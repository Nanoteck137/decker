// TODO(patrik):
//   - Add verbose printing
//   - Add documentation
//   - Cleanup the code
//
//   - Ubuntu:
//      - Install: libssl-dev musl-dev
//

use serde_json::Value;

use clap::{Parser, Subcommand};

use std::fs::File;
use std::io::{Write, Read};
use std::process::Command;
use std::path::{Path, PathBuf};

/// The helper program we send to the devkit
const DECKER_UTIL_PROGRAM: &[u8] = include_bytes!("../target/decker_util");

/// Custom error enum
#[derive(Debug)]
enum Error {
    /// Failed to send POST request to /register
    RegisterRequestFailed(reqwest::Error),

    /// Failed to get the text result from the POST request
    FailedToRetriveRegisterRequestText(reqwest::Error),

    /// Failed to parse the error json from the request
    FailedToParseErrorJson(serde_json::Error),

    /// Failed to register the host
    FailedToRegister(String),

    /// Failed to register but no message was supplied
    FailedToRegisterWithoutMessage,

    /// POST request with unknown status
    RegisterRequestUnknownStatus(u16),

    /// Failed to open the public key file
    FailedToOpenPublicKeyFile(std::io::Error),

    /// Failed to read the public key file
    FailedToReadPublicKeyFile(std::io::Error),

    /// Failed to execute 'ssh'
    FailedToExecuteSSH(std::io::Error),

    /// Failed to execute 'ssh-keygen'
    FailedToExecuteSSHKeygen(std::io::Error),

    /// Failed to execute 'scp'
    FailedToExecuteSCP(std::io::Error),

    /// Failed to execute 'rsync'
    FailedToExecuteRSync(std::io::Error),
}

/// Custom result type with our custom error enum
type Result<T> = std::result::Result<T, Error>;

/// Command line arguments the program accepts
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: ArgCommand,

    #[clap(short, value_parser)]
    devkit_addr: String,
}

/// Command line command
#[derive(Subcommand, Debug)]
enum ArgCommand {
    /// Deploy game
    Deploy {
        /// The game id the deployment should use
        #[clap(value_parser)]
        game_id: String,

        /// The program the deployment should run when the user runs the game
        #[clap(value_parser)]
        exec: String,

        /// The starting directory the deployment should start in when the
        /// user run the game
        #[clap(short, long, value_parser)]
        starting_dir: Option<String>,

        /// The directory on the host machine where the game files,
        /// so we can copy them to the devkit
        #[clap(value_parser)]
        game_file_dir: String,
    },

    /// Run shell
    Shell,
}

/// Get the path to the program data directory
fn get_data_dir() -> PathBuf {
    let mut res = dirs::data_local_dir().unwrap();
    res.push("decker");

    res
}

/// Get the path to the private key file
fn get_private_key_path() -> PathBuf {
    let mut res = get_data_dir();
    res.push("decker_devkit_key");

    res
}

/// Get the path to the public key file
fn get_public_key_path() -> PathBuf {
    let mut res = get_private_key_path();
    res.set_extension("pub");

    res
}

/// Read the public key and return the content
fn get_public_key() -> Result<String> {
    let path = get_public_key_path();

    let mut file =
        File::open(path).map_err(|e| Error::FailedToOpenPublicKeyFile(e))?;

    let mut result = String::new();
    file.read_to_string(&mut result)
        .map_err(|e| Error::FailedToReadPublicKeyFile(e))?;

    Ok(result)
}

/// Create the ssh keys needed for the devkit
fn create_ssh_keys() -> Result<()> {
    let path = get_private_key_path();

    Command::new("ssh-keygen")
        .arg("-f")
        .arg(path)
        .arg("-t")
        .arg("rsa")
        .arg("-b")
        .arg("2048")
        .arg("-N")
        .arg("")
        .output()
        .map_err(|e| Error::FailedToExecuteSSHKeygen(e))?;

    Ok(())
}

/// Register the host i.e send the ssh public key
fn register(addr: &str) -> Result<()> {
    let private_key = get_private_key_path();
    if !private_key.exists() {
        create_ssh_keys()?;
    }

    // TODO(patrik): Move port
    let url = format!("http://{}:32000/register", addr);

    let mut public_key = get_public_key()?;
    // TODO(patrik): We might not need to have the magic value because
    // registering without it works

    // Remove the newline
    public_key.pop();

    // NOTE(patrik): Magic from:
    // https://gitlab.steamos.cloud/devkit/steamos-devkit/-/blob/main/client/devkit_client/__init__.py
    // On Line: 872
    // Used in function: register

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

/// Execute command on the devkit
fn execute_simple_ssh(
    addr: &str,
    username: &str,
    cmd: &str,
) -> Result<std::process::Output> {
    let host = format!("{}@{}", username, addr);

    let key = get_private_key_path();

    Command::new("ssh")
        .arg("-oBatchMode=yes")
        .arg("-i")
        .arg(key)
        .arg(host)
        .arg(cmd)
        .output()
        .map_err(|e| Error::FailedToExecuteSSH(e))
}

/// Transfer files to/from the devkit
fn execute_simple_scp<S, D>(
    addr: &str,
    username: &str,
    source: S,
    dest: D,
) -> Result<std::process::Output>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    let host = format!("{}@{}", username, addr);

    let source = source.as_ref();
    let dest = dest.as_ref();
    let dest = format!("{}:{}", host, dest.to_str().unwrap());

    let key = get_private_key_path();

    Command::new("scp")
        .arg("-oBatchMode=yes")
        .arg("-i")
        .arg(key)
        .arg(source)
        .arg(dest)
        .output()
        .map_err(|e| Error::FailedToExecuteSCP(e))
}

/// Sync files from/to the devkit
fn execute_simple_rsync<S, D>(
    addr: &str,
    username: &str,
    source: S,
    dest: D,
) -> Result<std::process::Output>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    let host = format!("{}@{}", username, addr);

    let source = source.as_ref();
    let dest = dest.as_ref();
    let dest = format!("{}:{}", host, dest.to_str().unwrap());

    let key = get_private_key_path();

    Command::new("rsync")
        .arg("-e")
        .arg(format!("ssh -i \"{}\"", key.to_str().unwrap()))
        .arg("-r")
        .arg(source)
        .arg(dest)
        .output()
        .map_err(|e| Error::FailedToExecuteRSync(e))
}

/// Check if we already is registered
fn check_if_registered(addr: &str, username: &str) -> Result<bool> {
    let output = execute_simple_ssh(addr, username, "ls")?;

    Ok(output.status.success())
}

/// Debug print the output
fn _simple_print_output(output: &std::process::Output) {
    if output.status.success() {
        println!("{}", std::str::from_utf8(&output.stdout).unwrap());
    } else {
        println!("Error: {}", std::str::from_utf8(&output.stderr).unwrap());
    }
}

/// Deploy game to the devkit
fn deploy(
    addr: &str,
    username: &str,
    game_id: &str,
    exec: &str,
    starting_dir: &str,
    game_file_dir: &str,
) -> Result<()> {
    execute_simple_ssh(addr, username, "mkdir -p ~/decker")?;

    {
        let temp_file = mktemp::Temp::new_file().unwrap();
        let mut file = File::create(&temp_file).unwrap();
        file.write(DECKER_UTIL_PROGRAM).unwrap();

        execute_simple_scp(addr, username, temp_file, "~/decker/decker_util")?;
        execute_simple_ssh(addr, username, "chmod +x ~/decker/decker_util")?;
    }

    let cmd = format!("~/decker/decker_util prepare-upload {} true", game_id);
    let _output = execute_simple_ssh(addr, username, &cmd);
    // simple_print_output(&output);

    let cmd = format!(
        "~/decker/decker_util create-shortcut {} {} {}",
        game_id, exec, starting_dir
    );

    let _output = execute_simple_ssh(addr, username, &cmd)?;
    // TODO(patrik): Check for error from output
    // simple_print_output(&output);

    let mut game_file_dir = game_file_dir.to_string();
    if game_file_dir.chars().nth(game_file_dir.len() - 1).unwrap() != '/' {
        game_file_dir.push('/');
    }

    let source = game_file_dir;

    let dest = format!("~/decker-games/{}", game_id);

    let _output = execute_simple_rsync(addr, username, source, dest)?;
    // simple_print_output(&output);

    Ok(())
}

/// Run shell on the devkit
fn run_shell(addr: &str, username: &str) -> Result<()> {
    let host = format!("{}@{}", username, addr);

    let key = get_private_key_path();

    Command::new("ssh")
        .arg("-oBatchMode=yes")
        .arg("-oStrictHostKeyChecking=no")
        .arg("-i")
        .arg(key)
        .arg(host)
        .status()
        .map_err(|e| Error::FailedToExecuteSSH(e))?;

    Ok(())
}

/// Run the program
fn run(args: Args, addr: &str) -> Result<()> {
    let username = "deck";

    if !check_if_registered(addr, username)? {
        register(addr)?;
    }

    match args.command {
        ArgCommand::Deploy {
            game_id,
            exec,
            starting_dir,
            game_file_dir,
        } => {
            let exec = format!(
                "/home/{}/decker-games/{}/{}",
                username, game_id, exec
            );

            let starting_dir = if let Some(starting_dir) = starting_dir {
                starting_dir
            } else {
                format!("/home/{}/decker-games/{}", username, game_id)
            };

            deploy(
                &addr,
                username,
                &game_id,
                &exec,
                &starting_dir,
                &game_file_dir,
            )?;
        }

        ArgCommand::Shell => run_shell(&addr, "deck")?,
    }

    Ok(())
}

/// Entry point
fn main() -> Result<()> {
    let args = Args::parse();

    let addr = args.devkit_addr.clone();

    let path = get_data_dir();
    std::fs::create_dir_all(path).unwrap();

    println!("Device Address: {}", addr);

    run(args, &addr)
}
