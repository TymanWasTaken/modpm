use clap::{arg, Command};
use modpm::{ask_user, download_file, format_to_vec_of_strings, PolyMC};
use reqwest::Client;
use serde_json::Value;
use std::{env, error::Error};

fn cli() -> Command<'static> {
    Command::new("modpm")
        .about("A Minecraft mod package manager")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(false)
        .subcommand(
            Command::new("query")
                .about("Queries a mod")
                .arg(arg!(<MOD> "The mod to query."))
                .arg_required_else_help(true),
        )
        .subcommand(
            Command::new("download")
                .about("Downloads a mod")
                .arg(arg!(<MOD> "The mod to download."))
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("polymc").about("testing lmao"))
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("query", sub_matches)) => {
            let mmod = sub_matches.get_one::<String>("MOD").expect("required");
            let queried_mod = query_mod(mmod).await.unwrap();

            println!(
                "I found {}, with the ID {}.",
                queried_mod.name, queried_mod.id
            );
        }
        Some(("download", sub_matches)) => {
            let mmod = sub_matches.get_one::<String>("MOD").expect("required");
            let queried_mod = query_mod(mmod).await.unwrap();

            println!(
                "I found {}, with the ID {}.",
                queried_mod.name, queried_mod.id
            );

            let game_version = ask_user("What Minecraft version would you like? ");

            queried_mod.download(&game_version).await.unwrap();
        }
        Some(("polymc", _)) => {
            PolyMC::get_instances().unwrap();
        }
        _ => unreachable!(),
    }
}

async fn query_mod(mmod: &str) -> Result<Mod, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;

    let mod_string = format!("https://api.modrinth.com/v2/search?limit=1&facets=[[\"categories:fabric\"], [\"project_type:mod\"]]&query={}", mmod);
    let res = client.get(mod_string).send().await?;

    let body = res.text().await?;
    let json: Value = serde_json::from_str(&body[..])?;

    let new_mod = Mod::new(json).await?;

    Ok(new_mod)
}

#[derive(Debug)]
struct Mod {
    name: String,
    versions: Vec<String>,
    id: String,
}

impl Mod {
    async fn new(data: Value) -> Result<Mod, Box<dyn Error>> {
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

    async fn download(&self, game_version: &str) -> Result<(), Box<dyn Error>> {
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
}
