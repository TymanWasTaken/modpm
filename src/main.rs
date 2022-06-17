use modpm::{format_to_vec_of_strings, download_file};
use reqwest::Client;
use serde_json::Value;
use std::{
    env,
    error::Error,
    io::{stdin, stdout, Write},
    process
};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let action = &args.get(1).unwrap_or_else(|| {
        println!("help function");
        process::exit(1);
    });

    match &action[..] {
        "query" => {
            let mmod = &args.get(2).unwrap_or_else(|| {
                eprintln!("You need to input something for me to query.");
                process::exit(1);
            });
            query_mod(mmod).await.unwrap();
        }
        _ => process::exit(1),
    }
}

async fn query_mod(mmod: &str) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;

    let mod_string = 
    format!("https://api.modrinth.com/v2/search?limit=1&facets=[[\"categories:fabric\"], [\"project_type:mod\"]]&query={}", mmod);
    let res = client.get(mod_string).send().await?;

    let body = res.text().await?;
    let json: Value = serde_json::from_str(&body[..])?;

    let new_mod = Mod::new(json).await?;

    println!("Found mod {}, with the ID {}", new_mod.name, new_mod.id);

    new_mod.get_versions().await?;

    Ok(())
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

    async fn get_versions(&self) -> Result<(), Box<dyn Error>> {
        print!("what minecraft version do you want? ");
        stdout().flush().unwrap();

        let mut game_version = String::new();
        stdin().read_line(&mut game_version)?;
        game_version = game_version.replace("\n", "");

        let modrinth_versions_url = format!(
            "https://api.modrinth.com/v2/versions?ids={:?}",
            self.versions
        );

        let modrinth_versions_body = reqwest::get(modrinth_versions_url).await?.text().await?;

        let modrinth_versions: Value = serde_json::from_str(&modrinth_versions_body[..])?;

        for version in modrinth_versions.as_array().unwrap() {
            if format_to_vec_of_strings(version.get("game_versions").unwrap()).contains(&game_version)
            {

                let url = version["files"][0]["url"].as_str().unwrap();
                println!("{} matches the game version that you want", version.get("name").unwrap().as_str().unwrap());
                println!("Its download URL is {}", url);

                let reqwest_client = Client::new();

                download_file(&reqwest_client, url, "/home/skye/testing/examplefile").await?;
            }
        }

        Ok(())
    }
}
