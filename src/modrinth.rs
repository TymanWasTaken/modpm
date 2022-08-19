use crate::{data_structs::ModpmLockfile, PolyInstance};

use crate::{ask_user, crash, download_file, format_to_vec_of_strings, polymc::PolyMC, web_get};
use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModVersionFile {
    pub hashes: ModVersionFileHashes,
    pub url: String,
    pub filename: String,
    pub primary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModVersionFileHashes {
    pub sha512: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModVersionDependencies {
    pub version_id: Option<String>,
    pub project_id: Option<String>,
    pub dependency_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModVersion {
    pub mpm_id: Option<u8>,
    pub id: String,
    pub name: String,
    pub version_number: String,
    pub loaders: Vec<String>,
    pub files: Vec<ModVersionFile>,
    pub game_versions: Vec<String>,
    pub project_id: String,
    pub date_published: String,
    pub dependencies: Vec<ModVersionDependencies>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModrinthLicense {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModrinthDonationUrls {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModrinthTeamMember {
    pub team_id: String,
    pub user: ModrinthTeamUser,
    pub role: String,
    pub permissions: Option<u64>,
    pub accepted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

impl ModrinthTeamUser {
    pub fn display_name(&self) -> String {
        self.name.clone().unwrap_or(self.username.clone())
    }
}

impl MpmMod {
    pub async fn new(query: &str) -> Result<MpmMod, &str> {
        let data = web_get(&format!("https://api.modrinth.com/v2/project/{}", query)[..])
            .await
            .expect("Failed to get the mod data from Modrinth");

        if data.status().as_u16() == 404 {
            return Err("Couldn't find mod");
        }

        let json: serde_json::Value = json5::from_str(&data.text().await.unwrap()[..])
            .expect("Failed to turn the text into a JSON.");

        let title = json["title"].as_str().unwrap();
        let id = json["id"].as_str().unwrap();
        let license: ModrinthLicense = json5::from_str(&json["license"].to_string()[..]).unwrap();
        let versions: Vec<ModVersion> = json5::from_str(
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
            json5::from_str(&json["donation_urls"].to_string()[..]).unwrap();

        let team_url = format!("https://api.modrinth.com/v2/project/{}/members", id);

        let team_members_text = web_get(&team_url[..]).await.unwrap().text().await.unwrap();

        let members: Vec<ModrinthTeamMember> = json5::from_str(&team_members_text[..])
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
            crash("Couldn't get a mod's version from it's hash.".to_string());
        }

        let json: ModVersion = json5::from_str(&query.text().await.unwrap()[..]).unwrap();

        MpmMod::new(&json.project_id[..]).await.unwrap()
    }

    pub fn get_owner(&self) -> Option<&ModrinthTeamMember> {
        let members = &self.members;
        members.into_iter().find(|m| m.role == "Owner")
    }

    pub async fn download(&self, instance: PolyInstance, choose_version: bool) {
        let versions_base = &self.versions;

        // all the versions of the mod that work with the modloader and current game version
        let versions_filtered = versions_base.into_iter().filter(|v| {
            v.game_versions.contains(&instance.game_version)
                && v.loaders.contains(&instance.modloader)
        });

        // i blame rust for this
        let mut possible_versions: Vec<ModVersion> = vec![];

        for version in versions_filtered {
            possible_versions.push(version.clone());
        }

        // get the latest version's timestamp
        let mut latest_version_timestamp = 0;
        for version in &possible_versions {
            if version.time() > latest_version_timestamp {
                latest_version_timestamp = version.time()
            }
        }

        // get the latest *version*
        let latest_version = possible_versions
            .clone()
            .into_iter()
            .find(|v| v.time() == latest_version_timestamp)
            .unwrap();

        let version_to_download: ModVersion;

        let len;
        if choose_version {
            len = 2;
        } else {
            len = possible_versions.clone().len()
        }

        match len {
            // if there's no versions that work with the instance
            0 => {
                crash(format!(
                    "I couldn't find a version of {} that matches that instance.",
                    ansi_term::Color::Green.paint(&self.title)
                ));

                version_to_download = ModVersion {
                    mpm_id: None,
                    id: String::new(),
                    name: String::new(),
                    version_number: String::new(),
                    project_id: String::new(),
                    date_published: String::from("1970-01-01T00:00:00Z"),
                    files: vec![],
                    game_versions: vec![],
                    loaders: vec![],
                    dependencies: vec![],
                }
            }

            // if there's one version that works with the instance
            1 => version_to_download = possible_versions[0].clone(),

            // if there's any other number of versions that work with the instance
            _ => {
                let mut num = 0;
                let mut versions_with_id: Vec<ModVersion> = vec![];
                for mut v in possible_versions {
                    num += 1;
                    v.mpm_id = Some(num);
                    versions_with_id.push(v);
                }

                for version in &versions_with_id {
                    println!(
                        "{}: {} ({}{})",
                        version.mpm_id.expect("A mod version didn't have an ID"),
                        ansi_term::Color::Green.paint(&version.name),
                        ansi_term::Color::RGB(128, 128, 128).paint(&version.version_number),
                        ansi_term::Color::Red.paint(
                            if version.version_number == latest_version.version_number {
                                " latest"
                            } else {
                                ""
                            }
                        ),
                    );
                }

                let version_id = ask_user("What version of this mod do you want to download? ");

                version_to_download = versions_with_id
                    .into_iter()
                    .find(|i| i.mpm_id.unwrap().to_string() == version_id)
                    .expect("Couldn't find that instance.");
            }
        };

        if ModpmLockfile::get_lockfile(instance.clone())
            .into_iter()
            .find(|v| v.version.project_id == version_to_download.project_id)
            .is_some()
        {
            crash(format!("you've already downloaded that mod from modpm in this instance - if you wanted to update it, please run {}.", ansi_term::Color::RGB(128,128,128).paint("modpm update")));
        }

        MpmMod::download_specific_version(version_to_download.clone(), &instance).await;
    }

    #[async_recursion]
    pub async fn download_specific_version(version: ModVersion, instance: &PolyInstance) {
        let file_to_download: ModVersionFile = if version.files.len() == 1 {
            version.files[0].clone()
        } else {
            version
                .clone()
                .files
                .into_iter()
                .find(|f| f.primary == true)
                .expect("Couldn't find a mod version's primary file")
        };

        let path = format!(
            "{}/instances/{}/.minecraft/mods",
            PolyMC::get_directory(),
            instance.folder_name
        );

        let i_hate_rust = file_to_download.clone();

        println!("Downloading {}", file_to_download.filename);
        download_file(file_to_download.url, path, file_to_download.filename)
            .await
            .expect("Failed to download a mod file");

        ModpmLockfile::add_to_lockfile(instance.clone(), &version, &i_hate_rust);

        let mut deps: Vec<ModVersion> = vec![];

        for dep in version.dependencies {
            if dep.dependency_type == "required" {
                deps.push(
                    ModVersion::new({
                        if dep.version_id.is_some() {
                            dep.version_id.unwrap()
                        } else {
                            MpmMod::new(&dep.project_id.expect("A dependency didn't have a version ID or a project ID"))
                                .await
                                .expect("Couldn't fetch a dependency's project")
                                .versions
                                .into_iter()
                                .find(|v| v.loaders.contains(&instance.modloader) && v.game_versions.contains(&instance.game_version))
                                .expect("Couldn't find a dependency version that matches the PolyMC instance")
                                .id
                        }
                    })
                    .await,
                )
            }
        }

        for dep in deps {
            MpmMod::download_specific_version(dep, &instance).await
        }
    }
}

impl ModVersion {
    pub async fn new(id: String) -> ModVersion {
        let version_string = web_get(&format!("https://api.modrinth.com/v2/version/{}", id)[..])
            .await
            .expect("Couldn't get a version")
            .text()
            .await
            .expect("Couldn't get a version's text data");

        let version: ModVersion = json5::from_str(&version_string)
            .expect("Couldn't turn a version's JSON data into a ModVersion struct");

        version
    }
    pub fn time(&self) -> i64 {
        use chrono::prelude::*;

        let utc = DateTime::parse_from_rfc3339(&self.date_published[..]).unwrap();
        utc.timestamp()
    }
}
