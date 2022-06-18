use futures_util::StreamExt;
use progress_bar::{pb::ProgressBar, Color, Style};
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::io::{stdin, stdout};
use std::path::Path;
use std::string::String;
use std::{error::Error, fs::File, io::Write, usize};
use urlencoding::decode;

pub fn format_to_vec_of_strings(data: &Value) -> Vec<String> {
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

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), Box<dyn Error>> {
    let res = client.get(url).send().await.expect("failed to get the url");

    let filename = decode(res.url().path().split("/").last().unwrap()).unwrap();

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

pub fn polymc_installed() -> bool {
    let path = format!("{}/.local/share/PolyMC", env::var("HOME").unwrap());

    Path::new(&path).exists()
}

pub fn ask_user(query: &str) -> String {
    let mut response = String::new();
    print!("{}", query);
    stdout().flush().unwrap();

    stdin().read_line(&mut response).unwrap();
    response = response.replace("\n", "");

    response
}
