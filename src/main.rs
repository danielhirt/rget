use std::fs::File;
use std::io::Write;

use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

#[tokio::main]
async fn main() {
    let matches = Command::new("rget")
        .version("0.1.0")
        .author("Daniel Hirt <danielchirt16@gmail.com>")
        .about("wget clone written in Rust")
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .required(true)
                .help("URL to download"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .action(clap::ArgAction::SetTrue)
                .help("Suppress progress bar"),
        )
        .get_matches();

    let url = matches.get_one::<String>("url").unwrap();
    let quiet_mode = matches.get_flag("quiet");

    if let Err(e) = download(url, quiet_mode).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

async fn download(target: &str, quiet_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut response = client.get(target).send().await?.error_for_status()?;

    let length = response.content_length();
    let filename = extract_filename(target);

    let bar = create_progress_bar(quiet_mode, &filename, length);

    let mut file = File::create(&filename)?;

    while let Some(chunk) = response.chunk().await? {
        file.write_all(&chunk)?;
        bar.inc(chunk.len() as u64);
    }

    bar.finish_with_message(format!("{filename} downloaded"));
    Ok(())
}

fn extract_filename(url: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    match path.rsplit('/').next().filter(|name| !name.is_empty()) {
        Some(name) => name.to_string(),
        None => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap();
            format!("download_{}", now.as_secs())
        }
    }
}

fn create_progress_bar(quiet_mode: bool, msg: &str, length: Option<u64>) -> ProgressBar {
    let bar = match quiet_mode {
        true => ProgressBar::hidden(),
        false => match length {
            Some(len) => ProgressBar::new(len),
            None => ProgressBar::new_spinner(),
        },
    };

    bar.set_message(msg.to_string());

    match length.is_some() {
        true => bar.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        ),
        false => bar.set_style(ProgressStyle::default_spinner()),
    };

    bar
}
