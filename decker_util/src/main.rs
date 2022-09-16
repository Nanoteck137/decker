use std::path::Path;
use std::fs::File;
use std::io::{Write, Read};
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
    PrepareUpload {
        #[clap(value_parser)]
        game_id: String,

        #[clap(value_parser)]
        remove_old: bool,
    },
    Test,
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

fn read_file_binary<P>(path: P) -> Vec<u8>
where
    P: AsRef<Path>,
{
    let mut file = File::open(path).unwrap();

    let mut result = Vec::new();
    file.read_to_end(&mut result).unwrap();

    result
}

fn write_file_binary<P>(path: P, data: &Vec<u8>)
where
    P: AsRef<Path>,
{
    let mut file = File::create(path).unwrap();

    file.write(data).unwrap();
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

fn prepare_upload(game_id: String, remove_old: bool) {
    let mut path = std::env::current_dir().unwrap();
    path.push("devkit-game");
    path.push(game_id);

    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
        let path = path.to_str().unwrap();
        let data = serde_json::json!({
            "exists": false,
            "path": path,
        });

        print!("{}", data.to_string());
    } else {
        if remove_old {
            std::fs::remove_dir_all(&path).unwrap();
            std::fs::create_dir_all(&path).unwrap();
        }

        let path = path.to_str().unwrap();
        let data = serde_json::json!({
            "exists": true,
            "removed_old_content": remove_old,
            "path": path,
        });

        print!("{}", serde_json::to_string_pretty(&data).unwrap());
    }
}

fn test() {
    println!("Hello World");

    // let path = Path::new(".steam/steam.pipe");
    // let mut file = File::create(path).unwrap();
    // let cmd =
    //     format!("create-shortcut?response={}&gameid={}", "/tmp/wooh", "wooh");
    // let token = read_file(".steam/steam.token");
    // let pipe_cmd = format!("steam://devkit-1/{}/{}\n", token, cmd);
    // file.write(pipe_cmd.as_bytes()).unwrap();

    let path = Path::new(
        "/home/deck/.steam/steam/userdata/112778642/config/shortcuts.vdf",
    );
    let data = read_file_binary(&path);
    let mut obj = vdf::parse(&data).unwrap();
    println!("Obj: {:#?}", obj);
    if let vdf::Value::Object(obj) = obj.value_mut("Shortcuts").unwrap() {
        if let vdf::Value::Object(obj) = obj.value_mut("0").unwrap() {
            obj.set_value(
                "AppName".to_string(),
                vdf::Value::String("Testing for fun".to_string()),
            );
            println!("Obj: {:#?}", obj);
        }
    }

    let new_data = vdf::write(&obj).unwrap();
    write_file_binary(path, &new_data);
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Status => status(),
        Command::PrepareUpload {
            game_id,
            remove_old,
        } => prepare_upload(game_id, remove_old),
        Command::Test => test(),
    }
}
