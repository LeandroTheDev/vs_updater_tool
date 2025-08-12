use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use regex::Regex;

use crate::logger::LogsInstance;

pub struct Utils;

impl Utils {
    pub fn move_item(from: &Path, to: &Path) -> io::Result<()> {
        let from_path: &Path = from.as_ref();
        let to_dir: &Path = to.as_ref();

        let file_name = from_path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Invalid source path"))?;

        let to_path: PathBuf = to_dir.join(file_name);

        if let Some(parent) = to_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if to_path.exists() {
            if to_path.is_dir() {
                fs::remove_dir_all(&to_path)?;
            } else {
                fs::remove_file(&to_path)?;
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
            let _ = Utils::move_item(path.as_path(), to);
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
                    colored::Color::BrightRed,
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

        LogsInstance::print("Unknown system or game type", colored::Color::BrightRed);
        std::process::exit(1);
    }

    pub fn get_compress_type() -> String {
        if cfg!(target_os = "windows") {
            String::from(".zip")
        } else if cfg!(target_os = "linux") {
            String::from(".tar.gz")
        } else {
            LogsInstance::print("Unkown system", colored::Color::BrightRed);
            std::process::exit(1)
        }
    }

    pub fn url_exists(url: &str) -> bool {
        if cfg!(target_os = "windows") {
            Utils::url_exists_windows(url)
        } else if cfg!(target_os = "linux") {
            Utils::url_exists_linux(url)
        } else {
            LogsInstance::print("Unkown system", colored::Color::BrightRed);
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

    pub fn url_result(url: &str) -> Option<String> {
        if cfg!(target_os = "windows") {
            Utils::url_result_windows(url)
        } else if cfg!(target_os = "linux") {
            Utils::url_result_linux(url)
        } else {
            LogsInstance::print("Unkown system", colored::Color::BrightRed);
            std::process::exit(1)
        }
    }

    fn url_result_linux(url: &str) -> Option<String> {
        let output = Command::new("wget")
            .arg("-q") // Quiet mode, no logs
            .arg("-O") // Output for stdout
            .arg("-") // "-" stdout
            .arg(url)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    // Byte to string
                    String::from_utf8(result.stdout).ok()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    fn url_result_windows(url: &str) -> Option<String> {
        let output = Command::new("curl")
            .arg("-s") // Silent mode, no progress
            .arg(url)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    String::from_utf8(result.stdout).ok()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn extract_download_links(html: &str) -> Vec<String> {
        let re = Regex::new(r#"href="(/download/[^"]*)""#).unwrap();

        re.captures_iter(html)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .collect()
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
                    colored::Color::BrightRed,
                );
                return;
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
                                    colored::Color::BrightRed,
                                );
                                return;
                            }
                        }
                    } else {
                        return;
                    }
                }
                Err(e) => {
                    LogsInstance::print(
                        format!("Failed to read input: {}", e).as_str(),
                        colored::Color::BrightRed,
                    );
                    return;
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
            LogsInstance::print("Unkown system", colored::Color::BrightRed);
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
            if let Some(ext) = working_path.extension() {
                if ext.eq_ignore_ascii_case("zip") {
                    return Utils::uncompress_linux_zip(working_path);
                }
            }
            Utils::uncompress_linux(working_path)
        } else {
            LogsInstance::print("Unkown system", colored::Color::BrightRed);
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

    pub fn uncompress_linux_zip(compressed_version: &Path) -> Result<(), String> {
        if !compressed_version.exists() {
            return Err(format!(
                "File does not exist: {}",
                compressed_version.display()
            ));
        }

        let parent_dir = compressed_version
            .parent()
            .ok_or_else(|| "Failed to get parent directory".to_string())?;

        let status = Command::new("unzip")
            .arg(compressed_version.to_str().ok_or("Invalid file path")?)
            .arg("-d")
            .arg(parent_dir.to_str().ok_or("Invalid parent directory path")?)
            .status()
            .map_err(|e| format!("Failed to execute unzip: {}", e))?;

        if !status.success() {
            return Err(format!("unzip command failed with status: {}", status));
        }

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

    pub fn clear_temp(temp_dir: &Path, working_path: &Path) {
        match Utils::move_items(&temp_dir, &working_path) {
            Ok(_) => {}
            Err(e) => {
                LogsInstance::print(
                    format!("Failed to move temp to working path: {}", e).as_str(),
                    colored::Color::BrightRed,
                );
                return;
            }
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    pub fn get_mod_version(_: &PathBuf, mod_name: &OsStr) -> Option<String> {
        let filename: &str = mod_name.to_str()?;

        let trimmed: &str = if filename.ends_with(".zip") {
            filename.strip_suffix(".zip")?
        } else {
            filename
        };

        let last_underscore: usize = trimmed.rfind('_')?;

        let version: &str = &trimmed[last_underscore + 1..];

        return Some(version.to_string());
    }

    pub fn extract_id_and_filename(link: &str) -> Option<(String, String)> {
        // Expected: "/download/fileid/filename_fileversion.zip"
        let parts: Vec<&str> = link.split('/').collect();

        // Expected at least 4 parts: "", "download", "ID", "filename"
        if parts.len() >= 4 {
            let id: String = parts[2].to_string();
            let filename: String = parts[3].to_string();
            Some((id, filename))
        } else {
            None
        }
    }

    pub fn get_version_from_modinfo(modinfo_path: &PathBuf) -> Option<String> {
        let file: fs::File = fs::File::open(&modinfo_path).ok()?;
        let reader: BufReader<fs::File> = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                let line_trimmed = line.trim_start();
                // Compara a versão em lowercase para case-insensitive
                if line_trimmed.to_lowercase().starts_with("\"version\"") {
                    let parts: Vec<&str> = line_trimmed.split(':').collect();
                    if parts.len() >= 2 {
                        let version_raw = parts[1].trim().trim_matches(',').trim_matches('"');
                        return Some(version_raw.to_string());
                    }
                }
            }
        }

        None
    }

    pub fn get_mod_id(mod_path: &PathBuf) -> Option<String> {
        if mod_path.extension()? == "zip" {
            LogsInstance::print(
                "vs_updater does not support zip mods yet",
                colored::Color::BrightYellow,
            );
            return None;
        }

        let mod_id_path: PathBuf = mod_path.join("modid.txt");

        if !mod_id_path.exists() {
            LogsInstance::print(
                format!(
                    "The mod {} does not have a modid.txt and cannot be updated",
                    mod_path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("<unknown>")
                )
                .as_str(),
                colored::Color::BrightRed,
            );
            return None;
        }

        let contents: String = fs::read_to_string(mod_id_path).ok()?;

        // First line only
        let first_non_empty_line = contents
            .lines()
            .map(|line| line.trim())
            .find(|line| !line.is_empty())?;

        Some(first_non_empty_line.to_string())
    }

    pub fn get_mod_fileid(mod_path: &PathBuf) -> Option<String> {
        if mod_path.extension()? == "zip" {
            LogsInstance::print(
                "vs_updater does not support zip mods yet",
                colored::Color::BrightYellow,
            );
            return None;
        }

        let mod_id_path: PathBuf = mod_path.join("modid.txt");

        if !mod_id_path.exists() {
            LogsInstance::print(
                format!(
                    "The mod {} does not have a modid.txt and cannot be updated",
                    mod_path
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or("<unknown>")
                )
                .as_str(),
                colored::Color::BrightRed,
            );
            return None;
        }

        let contents: String = fs::read_to_string(mod_id_path).ok()?;

        // 2 Line only
        let mut non_empty_lines = contents
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty());

        non_empty_lines.nth(1).map(|line| line.to_string())
    }

    pub fn get_updated_path_from_version(
        path: &PathBuf,
        mod_name: &OsStr,
        new_version: &str,
    ) -> Option<PathBuf> {
        let filename: &str = match mod_name.to_str() {
            Some(name) => name,
            None => {
                LogsInstance::print(
                    format!("Invalid file name (UTF-8): {:?}", mod_name).as_str(),
                    colored::Color::BrightRed,
                );
                return None;
            }
        };

        let trimmed = if filename.ends_with(".zip") {
            match filename.strip_suffix(".zip") {
                Some(t) => t,
                None => {
                    LogsInstance::print(
                        format!("Error while removing the extension .zip from {}", filename)
                            .as_str(),
                        colored::Color::BrightRed,
                    );
                    return None;
                }
            }
        } else {
            filename
        };

        let last_underscore = match trimmed.rfind('_') {
            Some(i) => i,
            None => {
                LogsInstance::print(
                    format!("Mod does not contain '_' for versions: {}", trimmed).as_str(),
                    colored::Color::BrightRed,
                );
                return None;
            }
        };

        let prefix = &trimmed[..last_underscore];
        let new_name = format!("{}_{}", prefix, new_version);
        let new_path = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(&new_name);

        Some(new_path)
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
