use std::{
    env, fs,
    io::Write,
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
const MODS_URL: &str = "https://mods.vintagestory.at/";

fn main() {
    #[cfg(windows)]
    match colored::control::set_virtual_terminal(true) {
        Ok(_) => {}
        Err(_) => eprintln!("Cannot enable virtual terminal"),
    }

    let loaded_arguments: arguments::Items = arguments::Items::parse();

    if !loaded_arguments.ignore_game_update {
        update_game(&loaded_arguments);
    }

    if !loaded_arguments.ignore_mod_update && loaded_arguments.mods_path.is_some() {
        update_mods(&loaded_arguments);
    }
}

fn update_game(loaded_arguments: &arguments::Items) {
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
                colored::Color::BrightRed,
            );
            return;
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
            colored::Color::BrightRed,
        );

        return;
    }

    if let Some(folders) = &loaded_arguments.ignore_folders {
        for folder in folders {
            let full_folder_path: PathBuf = working_path.join(&folder);
            if let Err(e) = Utils::move_item(&full_folder_path, &temp_dir) {
                LogsInstance::print(
                    format!("Cannot move folder to temp: {}", e).as_str(),
                    colored::Color::BrightRed,
                );
                return;
            }
        }
    }

    if let Some(files) = &loaded_arguments.ignore_files {
        for file in files {
            let full_file_path: PathBuf = working_path.join(&file);
            if let Err(e) = Utils::move_item(&full_file_path, &temp_dir) {
                LogsInstance::print(
                    format!("Cannot move file to temp: {}", e).as_str(),
                    colored::Color::BrightRed,
                );
                return;
            }
        }
    }

    let mut game_version: GameVersion;
    if let Some(version) = Utils::get_game_version(&working_path) {
        game_version = match GameVersion::from_str(&version) {
            Some(ver) => ver,
            None => {
                LogsInstance::print(
                    format!("Invalid game version: {}", version).as_str(),
                    colored::Color::BrightRed,
                );
                return;
            }
        };
    } else {
        LogsInstance::print(
            "Unknown game version, add a file in assets/version-1.0.0.txt",
            colored::Color::BrightRed,
        );
        return;
    }

    let actual_game_version: GameVersion = game_version.clone();

    let game_type: String;
    if let Some(_type) = &loaded_arguments.game_type {
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

    let url_download: String;
    let mut last_version: GameVersion = GameVersion::from_str("0.0.0").unwrap();
    // If url is manually provided, get it
    if let Some(url) = &loaded_arguments.force_url {
        url_download = url.to_string();
    }
    // If not we try to get it
    else {
        loop {
            let ping_url: String = format!(
                "{}{}{}{}",
                MODS_URL,
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
            LogsInstance::print("No available versions found", colored::Color::BrightRed);
            Utils::clear_temp(&temp_dir, &working_path);
            return;
        }

        if last_version.equals(actual_game_version) {
            LogsInstance::print("No update needed! :D", colored::Color::BrightGreen);
            Utils::clear_temp(&temp_dir, &working_path);
            return;
        }

        url_download = format!(
            "{}{}{}{}",
            BASE_URL,
            game_type,
            last_version.to_string(),
            Utils::get_compress_type()
        );
    }

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
                colored::Color::BrightRed,
            );
            return;
        }
    }

    let compressed_version: PathBuf;
    match Utils::download_file(&url_download, &working_path) {
        Ok(path) => compressed_version = path,
        Err(e) => {
            LogsInstance::print(
                format!("Failed to download the version: {}", e).as_str(),
                colored::Color::BrightRed,
            );
            return;
        }
    }

    LogsInstance::print("File downloaded, decompressing...", colored::Color::White);

    match Utils::uncompress(&compressed_version) {
        Ok(_) => {}
        Err(e) => {
            LogsInstance::print(
                format!("Failed to uncompress: {}", e).as_str(),
                colored::Color::BrightRed,
            );
            return;
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

fn update_mods(loaded_arguments: &arguments::Items) {
    let mods_path = match &loaded_arguments.mods_path {
        Some(path) => path,
        None => {
            LogsInstance::print(
                "Ignoring mods update because the --mods-path is not set",
                colored::Color::Yellow,
            );
            return;
        }
    };

    // Getting zip mods
    {
        let path: &Path = Path::new(mods_path);

        if !path.is_dir() {
            LogsInstance::print("--mods-path is not valid", colored::Color::BrightRed);
            return;
        }

        let entries: fs::ReadDir = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(err) => {
                LogsInstance::print(
                    format!("Error reading directory: {}", err).as_str(),
                    colored::Color::BrightRed,
                );
                return;
            }
        };

        for entry_result in entries {
            let entry: fs::DirEntry = match entry_result {
                Ok(e) => e,
                Err(err) => {
                    LogsInstance::print(
                        format!("Error reading directory entry: {}", err).as_str(),
                        colored::Color::BrightRed,
                    );
                    continue;
                }
            };

            let path: PathBuf = entry.path();

            if let Some(name) = path.file_name() {
                LogsInstance::print("-----------------------------", colored::Color::BrightWhite);
                LogsInstance::print(
                    format!("Updating Mod: {}", name.to_string_lossy()).as_str(),
                    colored::Color::BrightWhite,
                );

                let (_, str_id, str_fileid) = match (
                    Utils::get_mod_version(&path, name),
                    Utils::get_mod_id(&path),
                    Utils::get_mod_fileid(&path),
                ) {
                    (Some(version), Some(id), Some(fileid)) => (version, id, fileid),
                    _ => {
                        LogsInstance::print(
                            format!("No modid.txt ignoring: {}", name.to_string_lossy()).as_str(),
                            colored::Color::BrightWhite,
                        );
                        continue;
                    }
                };

                let ping_url: String = format!("{}{}", MODS_URL, str_id);

                LogsInstance::print(
                    format!("Pinging: {}", ping_url).as_str(),
                    colored::Color::White,
                );

                let mut biggest_id: Option<i64> = None;
                let mut biggest_filename: Option<String> = None;

                if let Some(html) = Utils::url_result(&ping_url) {
                    let links: Vec<String> = Utils::extract_download_links(&html);
                    for link in links {
                        if let Some((id_str, filename)) = Utils::extract_id_and_filename(&link) {
                            if loaded_arguments.no_pre_mods {
                                if filename.contains("-pre") || filename.contains("-rc") {
                                    continue;
                                }
                            }
                            match id_str.parse::<i64>() {
                                Ok(id) => {
                                    biggest_filename = Some(filename);
                                    biggest_id = Some(match biggest_id {
                                        Some(current_max) => current_max.max(id),
                                        None => id,
                                    });
                                }
                                Err(_) => {
                                    LogsInstance::print(
                                        format!("Failed to parse id as integer: {}", id_str)
                                            .as_str(),
                                        colored::Color::BrightRed,
                                    );
                                }
                            }
                        } else {
                            LogsInstance::print(
                                format!("Invalid format link: {}", link).as_str(),
                                colored::Color::BrightRed,
                            );
                        }
                    }
                } else {
                    LogsInstance::print(
                        format!("Failed to get mod html: {}", mods_path).as_str(),
                        colored::Color::BrightRed,
                    );
                }

                let id_download: i64;
                let filename_download: String;
                if let Some(id) = biggest_id {
                    match str_fileid.parse::<i64>() {
                        Ok(actual_id) => {
                            if actual_id >= id {
                                LogsInstance::print(
                                    format!("Mod: {} is already on last version", ping_url)
                                        .as_str(),
                                    colored::Color::Green,
                                );
                                continue;
                            }
                        }
                        Err(_) => {}
                    }
                    id_download = id;
                } else {
                    LogsInstance::print(
                        format!("File id parse failed for: {}", mods_path).as_str(),
                        colored::Color::BrightRed,
                    );
                    continue;
                }

                if let Some(filename) = biggest_filename {
                    filename_download = filename;
                } else {
                    LogsInstance::print(
                        format!("File name parse failed for: {}", mods_path).as_str(),
                        colored::Color::BrightRed,
                    );
                    continue;
                }

                let url_download: String =
                    format!("{}download/{}/{}", MODS_URL, id_download, filename_download);

                LogsInstance::print(
                    format!("Mod update available: {}", url_download).as_str(),
                    colored::Color::BrightGreen,
                );

                if !Utils::url_exists(&url_download) {
                    LogsInstance::print(
                        "No connection or the mod does no longer exist",
                        colored::Color::BrightRed,
                    );
                    continue;
                }

                match Utils::clean_working_path(&path) {
                    Ok(_) => LogsInstance::print("Mod data removed", colored::Color::Green),
                    Err(e) => {
                        LogsInstance::print(
                            format!("Failed to clean mod data: {}", e).as_str(),
                            colored::Color::BrightRed,
                        );
                        continue;
                    }
                }

                let compressed_version: PathBuf;
                match Utils::download_file(&url_download, &path) {
                    Ok(path) => compressed_version = path,
                    Err(e) => {
                        LogsInstance::print(
                            format!("Failed to download the version: {}", e).as_str(),
                            colored::Color::BrightRed,
                        );
                        continue;
                    }
                }

                LogsInstance::print("File downloaded, decompressing...", colored::Color::White);

                match Utils::uncompress(&compressed_version) {
                    Ok(_) => {}
                    Err(e) => {
                        LogsInstance::print(
                            format!("Failed to uncompress: {}", e).as_str(),
                            colored::Color::BrightRed,
                        );
                        continue;
                    }
                }

                match fs::File::create(&path.join("modid.txt")) {
                    Ok(mut file) => {
                        if let Err(_) =
                            file.write_all(format!("{}\n{}", str_id, id_download).as_bytes())
                        {
                            LogsInstance::print(
                                "Cannot write mod id you will need to do manually",
                                colored::Color::BrightRed,
                            );
                        }
                    }
                    Err(_) => {
                        LogsInstance::print(
                            "Cannot create mod id file you will need to do manually",
                            colored::Color::BrightRed,
                        );
                    }
                }

                match fs::remove_file(&path.join(filename_download)) {
                    Ok(_) => {}
                    Err(_) => {
                        LogsInstance::print(
                            "Cannot delete mod update, you will need to delete it manually",
                            colored::Color::BrightRed,
                        );
                    }
                }

                let downloaded_version: String = match Utils::get_version_from_modinfo(
                    &path.join("modinfo.json"),
                ) {
                    Some(ver) => ver,
                    None => {
                        LogsInstance::print(
                                format!("Version not found in modinfo.json {}, version text will not be changed", path.display())
                                    .as_str(),
                                colored::Color::BrightYellow,
                            );
                        continue;
                    }
                };

                let new_path: PathBuf = match Utils::get_updated_path_from_version(
                    &path,
                    name,
                    downloaded_version.as_str(),
                ) {
                    Some(p) => p,
                    None => {
                        LogsInstance::print(
                            format!("Cannot get updated mod name: {}", path.display()).as_str(),
                            colored::Color::BrightRed,
                        );
                        continue;
                    }
                };

                match fs::rename(&path, &new_path) {
                    Ok(_) => LogsInstance::print(
                        "Successfully updated the mod",
                        colored::Color::BrightGreen,
                    ),
                    Err(_) => LogsInstance::print(
                        format!("Cannot rename the mod: {}", path.display()).as_str(),
                        colored::Color::BrightRed,
                    ),
                }
            }
        }
    }
}
