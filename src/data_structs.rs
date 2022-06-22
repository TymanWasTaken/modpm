use crate::{download_file, format_to_vec_of_strings, PolyMC};
use serde::Deserialize;
use serde_json::Value;
use std::{error::Error, process};

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Mod {
    pub name: String,
    pub versions: Vec<String>,
    pub id: String,
}

impl Mod {
    pub async fn new(data: Value) -> Result<Mod, Box<dyn Error>> {
        let mod_id = &data["hits"][0]["project_id"];
        let new_data_url =
            format!("https://api.modrinth.com/v2/project/{}", mod_id).replace("\"", "");
        let new_data_body = reqwest::get(new_data_url).await?.text().await?;
        let new_data: Value = serde_json::from_str(&new_data_body[..])?;
        let versions = new_data["versions"].clone();

        let new_versions = format_to_vec_of_strings(&versions);

        Ok(Mod {
            name: new_data["slug"].to_string().replace("\"", ""),
            versions: new_versions,
            id: new_data["id"].to_string().replace("\"", ""),
        })
    }

    pub async fn query(mmod: &str) -> Result<Mod, Box<dyn Error>> {
        let client = reqwest::Client::builder().build()?;

        let mod_string = format!(
            "https://api.modrinth.com/v2/search?limit=1&facets=[[\"project_type:mod\"]]&query={}",
            mmod
        );
        let res = client.get(mod_string).send().await?;

        let body = res.text().await?;
        let json: Value = serde_json::from_str(&body[..])?;

        let new_mod = Mod::new(json).await?;

        Ok(new_mod)
    }

    pub async fn download(&self, instance: PolyInstance) -> Result<(), Box<dyn Error>> {
        if instance.modloader == "vanilla" {
            eprintln!("I can't download mods to a vanilla instance.");
            process::exit(1);
        }

        let versions: Vec<ModVersion> = serde_json::from_str(
            &reqwest::get(format!(
                "https://api.modrinth.com/v2/versions?ids={:?}",
                self.versions
            ))
            .await
            .expect("Couldn't get the mod's versions info from Modrinth.")
            .text()
            .await
            .expect("Couldn't convert Modrinth version info into text.")[..],
        )
        .expect("Couldn't put Modrinth version data into a ModVersion vector.");

        let version = &versions
            .into_iter()
            .find(|v| {
                v.game_versions.contains(&instance.game_version)
                    && v.loaders.contains(&instance.modloader)
            })
            .expect(
                &format!(
                    "I couldn't find a version of {} that supports {}.",
                    self.name, instance.game_version,
                )[..],
            );

        let files = &version.files;
        let file = files
            .into_iter()
            .find(|f| f.primary)
            .expect("Couldn't find a primary file.");

        let path = format!(
            "{}/instances/{}/.minecraft/mods",
            PolyMC::get_directory(),
            instance.folder_name
        );
        println!("{:?}", file);
        println!("{}", path);

        download_file(
            &reqwest::Client::new(),
            &file.url[..],
            &path[..],
            &file.filename[..],
        )
        .await
        .expect("Failed to download the mod.");

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct ModVersion {
    name: String,
    version_number: String,
    loaders: Vec<String>,
    files: Vec<ModVersionFile>,
    game_versions: Vec<String>,
}
#[derive(Deserialize, Debug)]
struct ModVersionFile {
    hashes: ModVersionFileHashes,
    url: String,
    filename: String,
    primary: bool,
}
#[derive(Deserialize, Debug)]
struct ModVersionFileHashes {
    sha512: String,
    sha1: String,
}
