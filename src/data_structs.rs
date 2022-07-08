use crate::{crash, download_file, format_to_vec_of_strings, web_get, PolyMC};
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

#[derive(Deserialize, Debug)]
pub struct Mod {
    pub title: String,
    pub versions: Vec<String>,
    pub id: String,
}

impl Mod {
    pub async fn new(query: &str) -> Result<Mod, Box<dyn Error>> {
        let new_data_url =
            format!("https://api.modrinth.com/v2/project/{}", query).replace("\"", "");
        let new_data_body = web_get(&new_data_url).await?.text().await?;
        let new_data: Mod = serde_json::from_str(&new_data_body[..])?;

        Ok(new_data)
    }

    pub async fn download(&self, instance: PolyInstance) -> Result<(), Box<dyn Error>> {
        if instance.modloader == "vanilla" {
            eprintln!("I can't download mods to a vanilla instance.");
            process::exit(1);
        }

        let versions: Vec<ModVersion> = serde_json::from_str(
            &web_get(
                &format!(
                    "https://api.modrinth.com/v2/versions?ids={:?}",
                    self.versions
                )[..],
            )
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
                    self.title, instance.game_version,
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

        download_file(&file.url[..], &path[..], &file.filename[..])
            .await
            .expect("Failed to download the mod.");

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct ModVersion {
    pub loaders: Vec<String>,
    pub files: Vec<ModVersionFile>,
    pub game_versions: Vec<String>,
    pub project_id: String,
}
#[derive(Deserialize, Debug)]
pub struct ModVersionFile {
    pub hashes: ModVersionFileHashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
}
#[derive(Deserialize, Debug)]
pub struct ModVersionFileHashes {
    pub sha512: String,
    pub sha1: String,
}

#[derive(Debug)]
pub struct MpmMod {
    pub title: String,
    pub id: String,
    pub license: ModrinthLicense,
    pub versions: Vec<ModVersion>,
    pub description: String,
    pub categories: Vec<String>,
    pub source_url: String,
    pub donation_urls: Vec<ModrinthDonationUrls>,
    pub members: Vec<ModrinthTeamMember>,
}

#[derive(Deserialize, Debug)]
pub struct ModrinthLicense {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct ModrinthDonationUrls {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct ModrinthTeamMember {
    pub team_id: String,
    pub user: ModrinthTeamUser,
    pub role: String,
    pub permissions: Option<u64>,
    pub accepted: bool,
}

#[derive(Deserialize, Debug)]
pub struct ModrinthTeamUser {
    pub username: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub id: String,
    pub github_id: u64,
    pub avatar_url: String,
    pub created: String,
    pub role: String,
}

impl MpmMod {
    pub async fn new(query: &str) -> Result<MpmMod, &str> {
        let data = web_get(&format!("https://api.modrinth.com/v2/project/{}", query)[..])
            .await
            .expect("Failed to get the mod data from Modrinth");

        if data.status().as_u16() == 404 {
            return Err("Couldn't find mod");
        }

        let json: Value = serde_json::from_str(&data.text().await.unwrap()[..])
            .expect("Failed to turn the text into a JSON.");

        let title = json["title"].as_str().unwrap();
        let id = json["id"].as_str().unwrap();
        let license: ModrinthLicense =
            serde_json::from_str(&json["license"].to_string()[..]).unwrap();
        let versions: Vec<ModVersion> = serde_json::from_str(
            &web_get(
                &format!(
                    "https://api.modrinth.com/v2/versions?ids={:?}",
                    format_to_vec_of_strings(&json["versions"])
                )[..],
            )
            .await
            .expect("Couldn't get the mod's versions info from Modrinth.")
            .text()
            .await
            .expect("Couldn't convert Modrinth version info into text.")[..],
        )
        .expect("Couldn't put Modrinth version data into a ModVersion vector.");

        let description = json["description"].as_str().unwrap();
        let categories = format_to_vec_of_strings(&json["categories"]);
        let source_url = json["source_url"].as_str().unwrap();

        let donation_urls: Vec<ModrinthDonationUrls> =
            serde_json::from_str(&json["donation_urls"].to_string()[..]).unwrap();

        let team_url = format!("https://api.modrinth.com/v2/project/{}/members", id);

        let team_members_text = web_get(&team_url[..]).await.unwrap().text().await.unwrap();

        let members: Vec<ModrinthTeamMember> = serde_json::from_str(&team_members_text[..])
            .expect("Couldn't turn team members into the ModrinthTeamMember struct");

        Ok(MpmMod {
            title: title.to_string(),
            id: id.to_string(),
            license,
            versions,
            description: description.to_string(),
            categories,
            donation_urls,
            source_url: source_url.to_string(),
            members,
        })
    }

    pub async fn new_from_hash(hash: &str) -> MpmMod {
        let query_str = format!(
            "https://api.modrinth.com/v2/version_file/{}?algorithm=sha512",
            hash
        );

        let query = web_get(&query_str[..]).await.unwrap();
        if query.status().as_u16() == 404 {
            crash("Couldn't get a mod's version from it's hash.");
        }

        let json: ModVersion = serde_json::from_str(&query.text().await.unwrap()[..]).unwrap();

        MpmMod::new(&json.project_id[..]).await.unwrap()
    }

    pub async fn download(&self, instance: PolyInstance) {
        let versions = &self.versions;
        let version = versions.into_iter().filter(|v| {
            v.game_versions.contains(&instance.game_version)
                && v.loaders.contains(&instance.modloader)
        });

        println!("{:?}", version);
    }
}
