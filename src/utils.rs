use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::logger::LogsInstance;

pub struct Utils;

impl Utils {
    pub fn move_item<P: AsRef<Path>>(from: P, to: P) -> io::Result<()> {
        let from_path: &Path = from.as_ref();
        let to_dir: &Path = to.as_ref();

        let file_name: &std::ffi::OsStr = from_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid source path"))?;

        let to_path: PathBuf = to_dir.join(file_name);

        if let Some(parent) = to_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::rename(from_path, to_path)?;

        Ok(())
    }

    pub fn move_items(from: &Path, to: &Path) -> io::Result<()> {
        if !from.exists() || !from.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Invalid path"));
        }

        if !to.exists() || !to.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Invalid path"));
        }

        for entry_result in fs::read_dir(from)? {
            let entry = entry_result?;
            let path = entry.path();
            let _ = Utils::move_item(path, to.to_path_buf());
        }

        Ok(())
    }

    pub fn get_game_version(working_path: &Path) -> Option<String> {
        let assets_dir: PathBuf = working_path.join("assets");

        if !assets_dir.exists() || !assets_dir.is_dir() {
            return None;
        }

        let entries: fs::ReadDir = fs::read_dir(&assets_dir).ok()?;

        for entry in entries {
            let entry: fs::DirEntry = entry.ok()?;
            let file_name: std::ffi::OsString = entry.file_name();
            let file_name_str: std::borrow::Cow<'_, str> = file_name.to_string_lossy();

            if file_name_str.starts_with("version-") {
                let version: String = file_name_str.trim_start_matches("version-").to_string();
                return Some(version);
            }
        }

        None
    }

    pub fn get_game_type(game_type: &String) -> String {
        if cfg!(target_os = "windows") {
            if game_type == "client" {
                LogsInstance::print(
                    "This update tool does not support windows client update, because there is only .exe installer in official repositories",
                    colored::Color::Red,
                );
                std::process::exit(1);
            } else if game_type == "server" {
                return String::from("vs_server_win-x64_");
            }
        } else if cfg!(target_os = "linux") {
            if game_type == "client" {
                return String::from("vs_client_linux-x64_");
            } else if game_type == "server" {
                return String::from("vs_server_linux-x64_");
            }
        }

        LogsInstance::print("Unknown system or game type", colored::Color::Red);
        std::process::exit(1);
    }

    pub fn get_compress_type() -> String {
        if cfg!(target_os = "windows") {
            String::from(".zip")
        } else if cfg!(target_os = "linux") {
            String::from(".tar.gz")
        } else {
            LogsInstance::print("Unkown system", colored::Color::Red);
            std::process::exit(1)
        }
    }

    pub fn url_exists(url: &str) -> bool {
        if cfg!(target_os = "windows") {
            Utils::url_exists_windows(url)
        } else if cfg!(target_os = "linux") {
            Utils::url_exists_linux(url)
        } else {
            LogsInstance::print("Unkown system", colored::Color::Red);
            std::process::exit(1)
        }
    }

    fn url_exists_linux(url: &str) -> bool {
        let output = Command::new("wget")
            .arg("--spider") // Verify url option
            .arg(url)
            .arg("--quiet") // Supress logs
            .output();

        match output {
            Ok(result) => result.status.success(),
            Err(_) => false,
        }
    }

    fn url_exists_windows(url: &str) -> bool {
        let cmd: String = format!("Invoke-WebRequest -Uri '{}' -Method Head", url);
        let output: Result<std::process::Output, io::Error> =
            Command::new("powershell").args(["-Command", &cmd]).output();

        match output {
            Ok(result) => result.status.success(),
            Err(_) => false,
        }
    }

    pub fn check_temp_folder(working_path: &Path) {
        let temp_path = Path::new(working_path).join(".temp");

        if temp_path.exists() && temp_path.is_dir() {
            LogsInstance::print(
                "The folder .temp already exists, probably the updater tool exited before completing.",
                colored::Color::Yellow,
            );
            LogsInstance::print("Do you want to delete it? (y,N): ", colored::Color::Yellow);

            if let Err(e) = io::stdout().flush() {
                LogsInstance::print(
                    format!("Failed to flush stdout: {}", e).as_str(),
                    colored::Color::Red,
                );
                std::process::exit(1)
            }

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim().to_lowercase();
                    if input == "y" {
                        match fs::remove_dir_all(&temp_path) {
                            Ok(_) => {}
                            Err(e) => {
                                LogsInstance::print(
                                    format!("Failed to delete folder: {}", e).as_str(),
                                    colored::Color::Red,
                                );
                                std::process::exit(1)
                            }
                        }
                    } else {
                        std::process::exit(0)
                    }
                }
                Err(e) => {
                    LogsInstance::print(
                        format!("Failed to read input: {}", e).as_str(),
                        colored::Color::Red,
                    );
                    std::process::exit(1)
                }
            }
        }
    }

    pub fn countdown(seconds: u64, log_color: colored::Color) {
        for i in (1..=seconds).rev() {
            LogsInstance::print(
                format!(
                    "Continuing in {} second{}...",
                    i,
                    if i == 1 { "" } else { "s" }
                )
                .as_str(),
                log_color,
            );
            thread::sleep(Duration::from_secs(1));
        }
    }

    pub fn clean_working_path(working_path: &Path) -> io::Result<()> {
        let working_path: &Path = Path::new(working_path);
        let exe_path: PathBuf = std::env::current_exe().unwrap_or_default();
        let exe_canonical: PathBuf = exe_path.canonicalize().unwrap_or(exe_path.clone());

        for entry in fs::read_dir(working_path)? {
            let entry: fs::DirEntry = entry?;
            let path: PathBuf = entry.path();
            let item_name: std::ffi::OsString = entry.file_name();

            // Ignore .temp folder
            if item_name == ".temp" {
                continue;
            }

            // Ignore executable if exists in the working path
            let path_canonical: PathBuf = path.canonicalize().unwrap_or(path.clone());
            if path_canonical == exe_canonical {
                continue;
            }

            if path.is_dir() {
                if let Err(e) = fs::remove_dir_all(&path) {
                    LogsInstance::print(
                        format!("Failed to remove directory {}: {}", path.display(), e).as_str(),
                        colored::Color::Yellow,
                    );
                }
            } else {
                if let Err(e) = fs::remove_file(&path) {
                    LogsInstance::print(
                        format!("Failed to remove file {}: {}", path.display(), e).as_str(),
                        colored::Color::Yellow,
                    );
                }
            }
        }

        Ok(())
    }

    pub fn download_file(url: &str, working_path: &Path) -> Result<PathBuf, String> {
        if cfg!(target_os = "windows") {
            Utils::download_file_windows(url, working_path)
        } else if cfg!(target_os = "linux") {
            Utils::download_file_linux(url, working_path)
        } else {
            LogsInstance::print("Unkown system", colored::Color::Red);
            std::process::exit(1)
        }
    }

    fn download_file_linux(url: &str, working_path: &Path) -> Result<PathBuf, String> {
        let working_path: &Path = Path::new(working_path);
        if !working_path.exists() {
            return Err(format!(
                "Working path {} does not exist",
                working_path.display()
            ));
        }

        let file_name: &str = url.split('/').last().unwrap_or("invalid_file_name");
        let save_path: PathBuf = working_path.join(file_name);

        let status = Command::new("wget")
            .arg(url)
            .arg("-O")
            .arg(save_path.to_str().unwrap())
            .status()
            .map_err(|e| format!("Failed to execute wget: {}", e))?;

        if !status.success() {
            return Err(format!("Download failed with status: {}", status));
        }

        Ok(save_path)
    }

    fn download_file_windows(url: &str, working_path: &Path) -> Result<PathBuf, String> {
        let working_path: &Path = Path::new(working_path);
        if !working_path.exists() {
            return Err(format!(
                "Working path {} does not exist",
                working_path.display()
            ));
        }

        let file_name: &str = url.split('/').last().unwrap_or("invalid_file_name");
        let save_path: PathBuf = working_path.join(file_name);

        let status = Command::new("curl")
            .arg("-L")
            .arg(url)
            .arg("-o")
            .arg(save_path.to_str().unwrap())
            .status()
            .map_err(|e| format!("Failed to execute curl: {}", e))?;

        if !status.success() {
            return Err(format!("Download failed with status: {}", status));
        }

        Ok(save_path)
    }

    pub fn uncompress(working_path: &Path) -> Result<(), String> {
        if cfg!(target_os = "windows") {
            Utils::uncompress_windows(working_path)
        } else if cfg!(target_os = "linux") {
            Utils::uncompress_linux(working_path)
        } else {
            LogsInstance::print("Unkown system", colored::Color::Red);
            std::process::exit(1)
        }
    }

    pub fn uncompress_linux(compressed_version: &Path) -> Result<(), String> {
        if !compressed_version.exists() {
            return Err(format!(
                "File does not exist: {}",
                compressed_version.display()
            ));
        }

        let parent_dir: &Path = compressed_version
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let status = Command::new("tar")
            .arg("-xzf")
            .arg(compressed_version.to_str().ok_or("Invalid file path")?)
            .arg("-C")
            .arg(parent_dir.to_str().ok_or("Invalid parent directory")?)
            .status()
            .map_err(|e| format!("Failed to execute tar: {}", e))?;

        if !status.success() {
            return Err(format!("Tar command failed with status: {}", status));
        }

        let vintagestory_path: PathBuf = parent_dir.join("vintagestory");

        let _ = Utils::move_items(&vintagestory_path, parent_dir);

        Ok(())
    }

    pub fn uncompress_windows(compressed_version: &Path) -> Result<(), String> {
        if !compressed_version.exists() {
            return Err(format!(
                "File does not exist: {}",
                compressed_version.display()
            ));
        }

        let parent_dir: &Path = compressed_version
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let compressed_str: &str = compressed_version.to_str().ok_or("Invalid file path")?;

        let parent_str: &str = parent_dir.to_str().ok_or("Invalid parent directory")?;

        let status = Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
                compressed_str, parent_str
            ))
            .status()
            .map_err(|e| format!("Failed to execute Expand-Archive: {}", e))?;

        if !status.success() {
            return Err(format!(
                "Expand-Archive command failed with status: {}",
                status
            ));
        }

        let vintagestory_path: PathBuf = parent_dir.join("vintagestory");

        let _ = Utils::move_items(&vintagestory_path, parent_dir);

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GameVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl GameVersion {
    pub fn from_str(version: &str) -> Option<Self> {
        let cleaned = version.strip_suffix(".txt").unwrap_or(version);

        let parts: Vec<&str> = cleaned.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        Some(GameVersion {
            major: parts[0].parse().ok()?,
            minor: parts[1].parse().ok()?,
            patch: parts[2].parse().ok()?,
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    pub fn increment_patch(&mut self) {
        self.patch += 1;
    }

    pub fn increment_minor(&mut self) {
        self.patch = 0;
        self.minor += 1;
    }

    pub fn increment_major(&mut self) {
        self.patch = 0;
        self.minor = 0;
        self.major += 1;
    }

    pub fn equals(&mut self, game_version: GameVersion) -> bool {
        if self.major == game_version.major
            && self.minor == game_version.minor
            && self.patch == game_version.patch
        {
            return true;
        }
        return false;
    }

    pub fn empty(&mut self) -> bool {
        if self.major == 0 && self.minor == 0 && self.patch == 0 {
            return true;
        }
        return false;
    }
}
