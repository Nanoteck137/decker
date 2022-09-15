use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Status,
    PrepareUpload { game_id: String },
}

#[derive(Serialize, Deserialize)]
struct Status {
    id: String,
    name: String,
    pretty_name: String,
    build_id: String,
    variant_id: String,
    version_id: String,
    version_codename: String,
}

fn read_file<P>(path: P) -> String
where
    P: AsRef<Path>,
{
    let mut file = File::open(path).unwrap();

    let mut result = String::new();
    file.read_to_string(&mut result).unwrap();

    result
}

fn status() {
    let info = read_file("/etc/os-release");
    let info = info.trim_end();

    let mut map = HashMap::new();
    for split in info.split("\n") {
        let mut splits = split.split("=");
        let key = splits.next().unwrap();
        let value = splits.next().unwrap();

        let value = if &value[0..1] == "\"" {
            &value[1..value.len() - 1]
        } else {
            value
        };

        map.insert(key, value);
    }

    let status = Status {
        id: map.get("ID").unwrap().to_string(),
        name: map.get("NAME").unwrap().to_string(),
        pretty_name: map.get("PRETTY_NAME").unwrap().to_string(),
        build_id: map.get("BUILD_ID").unwrap().to_string(),
        variant_id: map.get("VARIANT_ID").unwrap().to_string(),
        version_id: map.get("VERSION_ID").unwrap().to_string(),
        version_codename: map.get("VERSION_CODENAME").unwrap().to_string(),
    };

    let s = serde_json::to_string_pretty(&status).unwrap();
    print!("{}", s);
}

fn prepare_upload(game_id: String) {
    println!("Prepare Upload: {}", game_id);
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Status => status(),
        Command::PrepareUpload { game_id } => prepare_upload(game_id),
    }
}
