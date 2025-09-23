use serde_derive::{Deserialize, Serialize};
use std::{fs, path::Path};

use anyhow::{anyhow, Result};

use crate::path::File;

const WALLSHIFT_DIR: &str = ".local/share/wallshift";

/// Returns the path to the current wallpaper information file
fn get_wallpaper_info_path() -> Result<String> {
    Ok(format!(
        "{}/{WALLSHIFT_DIR}/.current_wallpaper.yaml",
        get_home_dir()?,
    ))
}

fn get_home_dir() -> Result<String> {
    let home = home::home_dir()
        .ok_or(anyhow!("failed to get home directory"))?
        .to_str()
        .ok_or(anyhow!("failed to convert home directory to str"))?
        .to_owned();
    Ok(home)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileInfo {
    wallpaper: String,
    on: bool,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            wallpaper: String::default(),
            on: true,
        }
    }
}

/// Reads the YAML file and returns a FileInfo struct
fn read_config() -> Result<FileInfo> {
    std::fs::create_dir_all(format!(
        "{}/
        WALLSHIFT_DIR",
        get_home_dir()?
    ))?;

    let path_str = get_wallpaper_info_path()?;

    let path = Path::new(&path_str);

    let config: FileInfo = if path.exists() {
        let contents = fs::read_to_string(path)?;
        serde_yaml::from_str(&contents)?
    } else {
        FileInfo::default()
    };

    Ok(config)
}

/// Writes the FileInfo struct into the YAML file
fn write_config(config: &FileInfo) -> Result<()> {
    let path = get_wallpaper_info_path()?;
    let yaml = serde_yaml::to_string(config)?;
    fs::write(path, yaml)?;
    Ok(())
}

fn modify_config<F>(f: F) -> Result<()>
where
    F: Fn(FileInfo) -> FileInfo,
{
    let config = read_config()?;
    write_config(&f(config))?;
    Ok(())
}
/// Saves the path to the current wallpaper on the right file
pub fn save_wallpaper(wallpaper: &str) -> Result<()> {
    std::fs::create_dir_all(format!(
        "{}/
        WALLSHIFT_DIR",
        get_home_dir()?
    ))?;

    modify_config(|info| FileInfo {
        wallpaper: wallpaper.to_string(),
        ..info.clone()
    })
}

/// Gets the current wallpaper that has been stored on a particular config file.
pub fn get_current_wallpaper() -> Result<File> {
    let config = read_config()?;

    File::try_from(config.wallpaper)
        .map_err(|msg| anyhow!("failed to get current wallpaper: {msg}"))
}

pub fn is_on() -> Result<bool> {
    read_config().map(|c| c.on)
}

pub fn set_off() -> Result<()> {
    modify_config(|info| FileInfo {
        on: false,
        ..info.clone()
    })
}

pub fn set_on() -> Result<()> {
    modify_config(|info| FileInfo {
        on: true,
        ..info.clone()
    })
}
