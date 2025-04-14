use std::{
    env,
    path::{Path, PathBuf},
};

use clap::Parser;
use logger::LogsInstance;
use utils::GameVersion;
use utils::Utils;

mod arguments;
mod logger;
mod utils;

const BASE_URL: &str = "https://cdn.vintagestory.at/gamefiles/stable/";

fn main() {
    #[cfg(windows)]
    match colored::control::set_virtual_terminal(true) {
        Ok(_) => {}
        Err(_) => eprintln!("Cannot enable virtual terminal"),
    }

    let loaded_arguments: arguments::Items = arguments::Items::parse();

    // Getting work path
    let log_color: colored::Color;
    let working_path: PathBuf = if let Some(path) = &loaded_arguments.working_path {
        let path_buf: PathBuf = Path::new(path).to_path_buf();

        if path_buf.exists() && path_buf.is_dir() {
            log_color = colored::Color::Green;
            path_buf
        } else {
            LogsInstance::print(
                format!("The working-path: {}, is invalid", path).as_str(),
                colored::Color::Red,
            );
            std::process::exit(1);
        }
    } else {
        match env::var("VINTAGE_STORY") {
            Ok(value) => {
                log_color = colored::Color::BrightGreen;
                PathBuf::from(value)
            }
            Err(_) => {
                log_color = colored::Color::Yellow;

                env::current_exe()
                    .ok()
                    .and_then(|exe_path: PathBuf| exe_path.parent().map(|p: &Path| p.to_path_buf()))
                    .unwrap_or_else(|| PathBuf::from("."))
            }
        }
    };

    LogsInstance::print(
        format!("Working Directory: {:?}", working_path).as_str(),
        log_color,
    );

    Utils::check_temp_folder(&working_path);

    let temp_dir: PathBuf = working_path.join(".temp");

    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        LogsInstance::print(
            format!("Error creating temporary directory: {}", e).as_str(),
            colored::Color::Red,
        );

        std::process::exit(1);
    }

    if let Some(folders) = loaded_arguments.ignore_folders {
        for folder in folders {
            let full_folder_path: PathBuf = working_path.join(&folder);
            if let Err(e) = Utils::move_item(&full_folder_path, &temp_dir) {
                LogsInstance::print(
                    format!("Cannot move folder to temp: {}", e).as_str(),
                    colored::Color::Red,
                );
                std::process::exit(1);
            }
        }
    }

    if let Some(files) = loaded_arguments.ignore_files {
        for file in files {
            let full_file_path: PathBuf = working_path.join(&file);
            if let Err(e) = Utils::move_item(&full_file_path, &temp_dir) {
                LogsInstance::print(
                    format!("Cannot move file to temp: {}", e).as_str(),
                    colored::Color::Red,
                );
                std::process::exit(1);
            }
        }
    }

    let mut game_version: GameVersion;
    if let Some(version) = Utils::get_game_version(&working_path) {
        game_version = GameVersion::from_str(&version).unwrap_or_else(|| -> GameVersion {
            LogsInstance::print(
                format!("Invalid game version: {}", version).as_str(),
                colored::Color::Red,
            );
            std::process::exit(1)
        });
    } else {
        LogsInstance::print(
            "Unkown game version, add a file in assets/version-1.0.0.txt",
            colored::Color::Red,
        );
        std::process::exit(1);
    }
    let actual_game_version: GameVersion = game_version.clone();

    let game_type: String;
    if let Some(_type) = loaded_arguments.game_type {
        game_type = Utils::get_game_type(&_type);
    } else {
        game_type = Utils::get_game_type(&String::from("server"));
    }

    LogsInstance::print(
        format!(
            "Actual Version: {}{}{}",
            game_type,
            game_version.to_string(),
            Utils::get_compress_type()
        )
        .as_str(),
        colored::Color::White,
    );

    let mut last_version: GameVersion = GameVersion::from_str("0.0.0").unwrap();
    loop {
        let ping_url: String = format!(
            "{}{}{}{}",
            BASE_URL,
            game_type,
            game_version.to_string(),
            Utils::get_compress_type()
        );

        LogsInstance::print(
            format!("Pinging: {}", ping_url).as_str(),
            colored::Color::White,
        );

        if Utils::url_exists(&ping_url) {
            LogsInstance::print(
                format!("Version available: {}", game_version.to_string()).as_str(),
                colored::Color::Green,
            );
            last_version = game_version.clone();
            game_version.increment_patch();
        } else {
            if game_version.minor != actual_game_version.minor {
                if game_version.major != actual_game_version.major {
                    LogsInstance::print(
                        format!(
                            "Latest version available: {}, installed version: {}",
                            last_version.to_string(),
                            actual_game_version.to_string()
                        )
                        .as_str(),
                        colored::Color::BrightGreen,
                    );
                    break;
                } else {
                    game_version.increment_major();
                }
            } else {
                game_version.increment_minor();
            }
        }
    }

    if last_version.empty() {
        LogsInstance::print("No available versions found", colored::Color::Red);
        Utils::clear_temp(&temp_dir, &working_path);
        std::process::exit(1);
    }

    if last_version.equals(actual_game_version) {
        LogsInstance::print("No update needed! :D", colored::Color::BrightGreen);
        Utils::clear_temp(&temp_dir, &working_path);
        std::process::exit(0);
    }

    let url_download: String = format!(
        "{}{}{}{}",
        BASE_URL,
        game_type,
        last_version.to_string(),
        Utils::get_compress_type()
    );

    LogsInstance::print(
        format!(
            "All files and folders will be deleted in: {}, except for ignored!!",
            working_path.display()
        )
        .as_str(),
        colored::Color::BrightYellow,
    );

    Utils::countdown(5, colored::Color::BrightRed);

    match Utils::clean_working_path(&working_path) {
        Ok(_) => LogsInstance::print("Working path cleared!", colored::Color::Green),
        Err(e) => {
            LogsInstance::print(
                format!("Failed to clean working path: {}", e).as_str(),
                colored::Color::Red,
            );
            std::process::exit(1);
        }
    }

    let compressed_version: PathBuf;
    match Utils::download_file(&url_download, &working_path) {
        Ok(path) => compressed_version = path,
        Err(e) => {
            LogsInstance::print(
                format!("Failed to download the version: {}", e).as_str(),
                colored::Color::Red,
            );
            std::process::exit(1);
        }
    }

    LogsInstance::print("File downloaded, decompressing...", colored::Color::White);

    match Utils::uncompress(&compressed_version) {
        Ok(_) => {}
        Err(e) => {
            LogsInstance::print(
                format!("Failed to uncompress: {}", e).as_str(),
                colored::Color::Red,
            );
            std::process::exit(1);
        }
    }

    LogsInstance::print(
        "Moving temp files to working path...",
        colored::Color::White,
    );

    Utils::clear_temp(&temp_dir, &working_path);
    let _ = std::fs::remove_file(&compressed_version);

    LogsInstance::print(
        format!(
            "Success!!!, your vintage story has been updated to {}",
            last_version.to_string()
        )
        .as_str(),
        colored::Color::BrightGreen,
    );
}
