use crate::{download_file, format_to_vec_of_strings};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::{env, error::Error};

#[derive(Debug)]
pub struct PolyInstance {
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

    pub async fn download(&self, game_version: &str) -> Result<(), Box<dyn Error>> {
        let modrinth_versions_url = format!(
            "https://api.modrinth.com/v2/versions?ids={:?}",
            self.versions
        );

        let modrinth_versions_body = reqwest::get(modrinth_versions_url).await?.text().await?;

        let modrinth_versions: Value = serde_json::from_str(&modrinth_versions_body[..])?;

        for version in modrinth_versions.as_array().unwrap() {
            if format_to_vec_of_strings(version.get("game_versions").unwrap())
                .contains(&game_version.to_string())
            {
                let url = version["files"][0]["url"].as_str().unwrap();
                println!(
                    "{} matches the game version that you want",
                    version.get("name").unwrap().as_str().unwrap()
                );
                println!("Its download URL is {}", url);

                let reqwest_client = Client::new();

                download_file(
                    &reqwest_client,
                    url,
                    &format!("{}/Downloads", env::var("HOME")?)[..],
                )
                .await?;
            }
        }

        Ok(())
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
}
