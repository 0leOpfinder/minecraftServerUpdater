use std::fs;
use std::io::{self, Write};
use std::process::Command;
use reqwest;
use fs_extra;
use std::path::PathBuf;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the most current Minecraft version online
    let version_manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let response = reqwest::get(version_manifest_url).await?.text().await?;
    let version_manifest: Value = serde_json::from_str(&response)?;
    let latest_version = version_manifest["latest"]["release"].as_str().unwrap();
    println!("Latest version: {}", latest_version);

    // Get the download URL for Minecraft server files
    let server_download_page = "https://www.minecraft.net/da-dk/download/server"; // i think "en-us" is the official url. i'v choosen da-dk just because... well... i'm from denmark 
    let response = reqwest::get(server_download_page).await?.text().await?;
    let download_url = response
        .lines()
        .find(|line| line.contains("jar") && line.contains("href"))
        .and_then(|line| {
            let start = line.find("href=\"").map(|index| index + 6)?;
            let end = line[start..].find('"').map(|index| index + start)?;
            Some(&line[start..end])
        })
        .expect("Could not find Minecraft server download link");
    println!("Download URL: {}", download_url);

    // Load old version from mc_version.txt
    let old_version = fs::read_to_string("mc_version.txt").unwrap_or_default();
    println!("The current version being used is: {}", old_version);

    // Compare release versions and update if needed
    if old_version.trim() != latest_version {
        // Back up the MC_Server folder
        let user_profile = dirs::home_dir().expect("Could not find user profile directory");
        //let mc_server_path = user_profile.join("Desktop/MC_Server");
        //let backup_path = user_profile.join("Desktop/backup/MC_Server");
        let mc_server_path = PathBuf::from("./");
        let backup_path = PathBuf::from("./backup");

        if mc_server_path.is_dir() {
            fs::create_dir_all(&backup_path)?;
            fs_extra::dir::copy(&mc_server_path, &backup_path, &fs_extra::dir::CopyOptions::new())?;
            println!("Backup done.");
        } else {
            println!("No existing server folder to back up.");
        }

        // Replace the version number in mc_version.txt
        fs::write("mc_version.txt", latest_version)?;

        // Download the latest version of Minecraft
        println!("Downloading latest Minecraft server jar...");
        let server_jar_response = reqwest::get(download_url).await?.bytes().await?;
        fs::write("server.jar", server_jar_response)?;
        println!("Download done.");
    } else {
        println!("Minecraft server is already up to date.");
    }

    Ok(())
}
