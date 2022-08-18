use crate::{parse_cfg_file, crash, data_structs::ModpmLockfile};
use std::{error::Error, path::Path, env, fs};
use serde_derive::{Serialize, Deserialize};

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
                crash("The OS is linux, but neither the default nor the flatpak PolyMC folder locations could be found".to_string())
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


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolyInstance {
    pub id: u32,
    pub name: String,
    pub folder_name: String,
    pub game_version: String,
    pub modloader: String,
}
#[derive(Deserialize, Debug)]
pub struct PolyInstanceDataComponent {
    pub uid: String,
    pub version: String,
}
#[derive(Deserialize)]
pub struct PolyInstanceDataJson {
    pub components: Vec<PolyInstanceDataComponent>,
}

impl PolyInstance {
    pub fn update(&self) -> Result<(), Box<dyn Error>> {
        let lockfile = ModpmLockfile::get_lockfile(self.clone());

        for ver in lockfile {
            println!("{:?}\n", ver);
        }

        Ok(())
    }
}
