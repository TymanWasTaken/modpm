pub mod data_structs;
pub mod modrinth;
pub mod polymc;

use polymc::PolyInstance;

use futures_util::StreamExt;
use progress_bar::{pb::ProgressBar, Color, Style};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use std::fs;
use std::io::{self, stdin, stdout};
use std::string::String;
use std::{error::Error, fs::File, io::Write, process, usize};

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

pub async fn download_file(
    url: String,
    path: String,
    filename: String,
) -> Result<(), Box<dyn Error>> {
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

pub fn crash(reason: String) -> String {
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
