use std::fs;
use reqwest;
use std::path::PathBuf;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ==== Get the most current Minecraft version online ==========================================
    let version_manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let response = reqwest::get(version_manifest_url).await?.text().await?;
    let version_manifest: Value = serde_json::from_str(&response)?;
    let latest_version = version_manifest["latest"]["release"].as_str().unwrap();
    println!("Latest version: {}", latest_version);
    // ==============================================================================================

    // ==== Get the download URL for Minecraft server files =========================================
    let server_download_page = "https://www.minecraft.net/da-dk/download/server";
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
    // ==============================================================================================

    // Get which version we are currently running from mc_version.txt
    let old_version = fs::read_to_string("mc_version.txt").unwrap_or_default();

    // ==== Compare with release versions and update if needed ======================================
    if old_version.trim() != latest_version { 

        // ---- Back up the server.jar file and its version -----------------------------------------
        let backup_file_name = if old_version.trim().is_empty() {
            "backup/server_old.jar"
        } else {
            &format!("backup/server_{}.jar", old_version.trim())
        };
        let backup_path = PathBuf::from(backup_file_name);

        if PathBuf::from("server.jar").exists() {
            fs::create_dir_all("backup")?;
            fs::copy("server.jar", &backup_path)?;
            println!("Backup of server.jar created: {}", backup_path.display());
        } else {
            println!("No existing server.jar to back up.");
        }
        // ------------------------------------------------------------------------------------------

        // ---- Get the new version -----------------------------------------------------------------
        fs::write("mc_version.txt", latest_version)?;

        // Download the latest version of Minecraft
        println!("Downloading latest Minecraft server jar...");
        let server_jar_response = reqwest::get(download_url).await?.bytes().await?;
        fs::write("server.jar", server_jar_response)?;
        println!("Download done.");
        // ------------------------------------------------------------------------------------------
    } else {
        println!("Minecraft server is already up to date.");
    }
    // ==============================================================================================

    Ok(())
}
