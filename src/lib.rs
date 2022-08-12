pub mod data_structs;
pub mod polymc;
pub mod modrinth;

use polymc::{PolyInstance, PolyInstanceDataJson};

use futures_util::StreamExt;
use progress_bar::{pb::ProgressBar, Color, Style};
use std::collections::HashMap;
use std::io::{stdin, stdout, self};
use std::path::Path;
use std::string::String;
use std::{env, fs};
use std::{error::Error, fs::File, io::Write, usize, process};
use sha2::{Sha512, Digest};

async fn web_get(url: &str) -> Result<reqwest::Response, reqwest::Error> {
    reqwest::Client::new()
        .get(url)
        .header(
            reqwest::header::USER_AGENT,
            "modpm/0.1.0 (https://github.com/Lisenaaaa/modpm)",
        )
        .send()
        .await
}

pub fn format_to_vec_of_strings(data: &serde_json::Value) -> Vec<String> {
    let mut new_data: Vec<String> = vec![];

    if data.is_array() {
        for items in data.as_array() {
            for item in items {
                new_data.push(item.to_string().replace("\"", ""));
            }
        }
    }

    new_data
}

pub async fn download_file(url: String, path: String, filename: String) -> Result<(), Box<dyn Error>> {
    let res = web_get(&url[..]).await.expect("failed to get the url");

    let total_size = res
        .content_length()
        .expect("failed to get the content length");

    let mut pb = ProgressBar::new(usize::try_from(total_size)?);
    pb.set_action("Downloading", Color::LightGreen, Style::Normal);

    let mut file =
        File::create(format!("{}/{}", path, filename)).expect("failed to create the file");
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.expect("error while downloading file");

        file.write_all(&chunk).expect("error while writing to file");

        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_progression(usize::try_from(new)?);
    }

    pb.finalize();

    Ok(())
}

pub fn ask_user(query: &str) -> String {
    let mut response = String::new();
    print!("{}", query);
    stdout().flush().unwrap();

    stdin().read_line(&mut response).unwrap();
    
    response.trim().to_string()
}

pub fn parse_cfg_file(filepath: String) -> HashMap<String, String> {
    let file = fs::read_to_string(filepath).unwrap();
    let file_split: Vec<&str> = file.split("\n").filter(|c| *c != "").collect();

    let mut map: HashMap<String, String> = HashMap::new();

    for data in file_split {
        let split_data: Vec<&str> = data.split("=").collect();

        map.insert(split_data[0].to_string(), split_data[1].to_string());
    }

    map
}

pub struct PolyMC {}

impl PolyMC {
    pub fn get_directory() -> String {
        match std::env::consts::OS {
            "linux" => {
                let home_dir = env::var("HOME").expect("Couldn't get the $HOME env var.");
                // Check if the main dir (~/.local/share/PolyMC) exists
                let main_dir = format!("{}/.local/share/PolyMC", home_dir);
                let main_dir = Path::new(&main_dir);
                if main_dir.exists() {
                    return main_dir
                        .to_str()
                        .expect("Unable to convert Path instance to &str")
                        .to_string();
                }
                // Check if the old main dir (~/.local/share/polymc) exists
                let old_main_dir = format!("{}/.local/share/polymc", home_dir);
                let old_main_dir = Path::new(&old_main_dir);
                if old_main_dir.exists() {
                    return old_main_dir
                        .to_str()
                        .expect("Unable to convert Path instance to &str")
                        .to_string();
                }
                // Otherwise, check for the flatpak directory
                let flatpak_dir = &format!("{}/.var/app/org.polymc.PolyMC/data/PolyMC", home_dir);
                let flatpak_dir = Path::new(&flatpak_dir);
                if flatpak_dir.exists() {
                    return flatpak_dir
                        .to_str()
                        .expect("Unable to convert Path instance to &str")
                        .to_string();
                }
                crash("The OS is linux, but neither the default nor the flatpak PolyMC folder locations could be found")
            }
            "macos" => {
                return format!(
                    "{}/Library/Application Support/PolyMC",
                    env::var("HOME").expect("Couldn't get the $HOME env var.")
                )
            }
            "windows" => {
                // windows <:hollow:829582572983943209>
                // this os has so many problems with it i stg
                return format!(
                    "{}\\AppData\\Roaming\\PolyMC",
                    env::var("HOME").expect("Couldn't get the $HOME env var.")
                )
            }
            _ => {
                return format!(
                    "{}/.local/share/PolyMC",
                    env::var("HOME").expect("Couldn't get the $HOME env var.")
                )
            }
        }
    }

    pub fn is_installed() -> bool {
        let path = PolyMC::get_directory();

        Path::new(&path).exists()
    }

    pub fn get_instances() -> Result<Vec<PolyInstance>, Box<dyn Error>> {
        let poly_dir = PolyMC::get_directory();

        let mut return_instances: Vec<PolyInstance> = vec![];
        let mut num = 0;
        let instance_dirs_wtf = fs::read_dir(&format!("{}/instances", poly_dir))?;
        let mut instance_dirs = vec![];
        for dir in instance_dirs_wtf {
            instance_dirs.push(dir.unwrap());
        }

        instance_dirs = instance_dirs
            .into_iter()
            .filter(|t| {
                t.file_name() != ".LAUNCHER_TEMP"
                    && t.file_name() != "_LAUNCHER_TEMP"
                    && t.file_type().unwrap().is_dir()
            })
            .collect();

        for dir in instance_dirs {
            num = num + 1;
            let instance_config = parse_cfg_file(format!("{}/instance.cfg", dir.path().display()));
            let mmc_pack: PolyInstanceDataJson = serde_json::from_str(
                &fs::read_to_string(format!("{}/mmc-pack.json", dir.path().display()))
                    .expect("Failed to read the JSON data for a PolyMC instance.")[..],
            )
            .expect("Failed to parse the JSON data for a PolyMC instance.");

            let instance_components = &mmc_pack.components;
            let game_version = &instance_components
                .into_iter()
                .find(|c| c.uid == "net.minecraft")
                .expect("Couldn't find a Minecraft component in a PolyMC instance.")
                .version;

            let modloader_id_option = instance_components.into_iter().find(|c| {
                c.uid == "net.fabricmc.fabric-loader"
                    || c.uid == "org.quiltmc.quilt-loader"
                    || c.uid == "net.minecraftforge"
            });

            let instance_name = instance_config
                .get("name")
                .expect("A PolyMC instance.cfg didn't have a name field.");

            let modloader_id = match &modloader_id_option {
                Some(modloader_id) => PolyMC::get_loader_name(&modloader_id.uid)
                    .expect("Unable to determine loader name from uid"),
                None => "vanilla",
            };

            return_instances.push(PolyInstance {
                id: num,        
                name: instance_name.to_string(),
                modloader: modloader_id.to_string(),
                game_version: game_version.to_string(),
                folder_name: dir.file_name().to_str().expect("something went wrong when converting an OsString to a String lmao i have no idea how this went wrong").to_string(),
            });
        }

        Ok(return_instances)
    

    }

    pub fn get_loader_name(uid: &str) -> Option<&str> {
        match uid {
            "net.fabricmc.fabric-loader" => Some("fabric"),
            "org.quiltmc.quilt-loader" => Some("quilt"),
            "net.minecraftforge" => Some("forge"),
            _ => None,
        }
    }
}

pub fn crash(reason: &str) -> String {
    eprintln!("{}", reason);
    process::exit(1);
}


pub fn hash_file(path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(path).unwrap();
    let mut hasher = Sha512::new();
    io::copy(&mut file, &mut hasher).unwrap();
    let result = hasher.finalize();

    let hash = hex::encode(result);

    Ok(hash)
}
