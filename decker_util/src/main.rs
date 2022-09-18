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
    CreateShortcut {
        #[clap(value_parser)]
        game_id: String,

        #[clap(value_parser)]
        exec: String,

        #[clap(value_parser)]
        starting_dir: String,
    },
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
    path.push("decker-games");
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

fn gen_id(exe: &String, app_name: &String) -> u32 {
    let crc = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    let key = exe.clone() + app_name;

    crc.checksum(key.as_bytes()) | 0x80000000
}

fn create_shortcut_obj(
    id: u32,
    game_id: String,
    app_name: String,
    exec: String,
    starting_dir: String,
) -> vdf::Object {
    let mut new_obj = vdf::Object::new();

    new_obj.set_value("appid".to_string(), vdf::Value::Integer(id));

    new_obj.set_value("AppName".to_string(), vdf::Value::String(app_name));

    new_obj.set_value("Exe".to_string(), vdf::Value::String(exec));

    new_obj
        .set_value("StartDir".to_string(), vdf::Value::String(starting_dir));

    new_obj.set_value("icon".to_string(), vdf::Value::String("".to_string()));

    new_obj.set_value(
        "ShortcutPath".to_string(),
        vdf::Value::String("".to_string()),
    );

    new_obj.set_value(
        "LaunchOptions".to_string(),
        vdf::Value::String("".to_string()),
    );

    new_obj.set_value("IsHidden".to_string(), vdf::Value::Integer(0));

    new_obj
        .set_value("AllowDesktopConfig".to_string(), vdf::Value::Integer(1));

    new_obj.set_value("AllowOverlay".to_string(), vdf::Value::Integer(1));

    new_obj.set_value("OpenVR".to_string(), vdf::Value::Integer(0));

    new_obj.set_value("Devkit".to_string(), vdf::Value::Integer(1));

    new_obj.set_value(
        "DevkitGameID".to_string(),
        vdf::Value::String(game_id.clone()),
    );

    new_obj
        .set_value("DevkitOverrideAppID".to_string(), vdf::Value::Integer(0));

    new_obj.set_value(
        "LastPlayTime".to_string(),
        vdf::Value::Integer(1663261394),
    );

    new_obj.set_value(
        "FlatpakAppID".to_string(),
        vdf::Value::String("".to_string()),
    );

    let tags = vdf::Object::new();
    new_obj.set_value("tags".to_string(), vdf::Value::Object(tags));

    new_obj
}

fn update_shortcut(_obj: &mut vdf::Object) {
    // TODO(patrik): Update shortcut
    println!("TODO: Update shortcut");
}

fn create_shortcut(game_id: String, exec: String, starting_dir: String) {
    println!("Creating shortcut: {}", game_id);

    let app_name = format!("Decker: {}", game_id);

    let id = gen_id(&exec, &app_name);
    println!("ID: {}", id);

    let path = Path::new("/home/deck/.steam/steam/userdata/");
    for dir in std::fs::read_dir(path).unwrap() {
        let mut path = dir.unwrap().path();
        path.push("config/shortcuts.vdf");

        if !path.exists() {
            File::create(&path).unwrap();
        }

        let data = read_file_binary(&path);
        let mut obj = vdf::parse(&data).unwrap();
        // println!("Obj: {:#?}", obj);

        let shortcuts = obj.value_mut("Shortcuts");
        if shortcuts.is_none() {
            obj.set_value(
                "Shortcuts".to_string(),
                vdf::Value::Object(vdf::Object::new()),
            );
        }

        if let vdf::Value::Object(obj) = obj.value_mut("Shortcuts").unwrap() {
            let mut found = false;

            for value in obj.values_mut().iter_mut() {
                if let vdf::Value::Object(obj) = &mut value.1 {
                    let appid = obj.value("appid").unwrap();
                    if let vdf::Value::Integer(appid) = appid {
                        if *appid == id {
                            update_shortcut(obj);
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                let mut max = 0;
                for value in obj.values() {
                    let index = value.0.parse::<u32>().unwrap();
                    max = index.max(max);
                }

                let new_index = max + 1;

                let new_obj = create_shortcut_obj(
                    id,
                    game_id.clone(),
                    app_name.clone(),
                    exec.clone(),
                    starting_dir.clone(),
                );

                obj.set_value(
                    new_index.to_string(),
                    vdf::Value::Object(new_obj),
                );
            }
        }

        let new_data = vdf::write(&obj).unwrap();
        write_file_binary(path, &new_data);
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Status => status(),
        Command::PrepareUpload {
            game_id,
            remove_old,
        } => prepare_upload(game_id, remove_old),
        Command::CreateShortcut {
            game_id,
            exec,
            starting_dir,
        } => create_shortcut(game_id, exec, starting_dir),
    }
}
