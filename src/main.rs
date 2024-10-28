use std::fs;
use reqwest;
use std::path::PathBuf;
use serde_json::Value;
use std::process::Command;
use std::{thread, time};

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

        // ---- Stop the current minecraft server from running --------------------------------------
        stop_minecraft_server();
        wait_for_minecraft_shutdown();
        // ------------------------------------------------------------------------------------------

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

        // ---- Start minecraft server up again -----------------------------------------------------
        start_minecraft_server();
        // ------------------------------------------------------------------------------------------
    } else {
        println!("Minecraft server is already up to date.");
    }
    // ==============================================================================================

    Ok(())
}

fn stop_minecraft_server() {
    // Assuming your Minecraft server is running in a screen session named "minecraft"
    let status = Command::new("screen")
        .args(&["-S", "minecraft", "-X", "stuff", "stop\n"])
        .status()
        .expect("Failed to stop Minecraft server");
    if status.success() {
        println!("Minecraft server stopped successfully.");
    } else {
        println!("Failed to stop Minecraft server.");
    }
}

fn start_minecraft_server() {
    let status = Command::new("screen")
        .args(&["-dmS", "minecraft", "java", "-Xmx1024M", "-Xms1024M", "-jar", "server.jar", "nogui"])
        .status()
        .expect("Failed to start Minecraft server");
    if status.success() {
        println!("Minecraft server started successfully.");
    } else {
        println!("Failed to start Minecraft server.");
    }
}

fn wait_for_minecraft_shutdown() {
    // Poll every 5 seconds to check if the Minecraft server is still running
    let wait_time = time::Duration::from_secs(5);
    loop {
        let result = Command::new("screen")
            .args(&["-ls"])
            .output()
            .expect("Failed to list screen sessions");
        let output = String::from_utf8_lossy(&result.stdout);

        // Check if the "minecraft" session is still listed
        if !output.contains("minecraft") {
            println!("Minecraft server has fully shut down.");
            break;
        }

        println!("Waiting for Minecraft server to shut down...");
        thread::sleep(wait_time);
    }
}


