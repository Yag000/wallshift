use rand::Rng;
use std::{
    fs::{read_dir, read_to_string, DirEntry},
    path::PathBuf,
    process::Command,
};

use crate::{
    configuration::Settings,
    path::{File, ImagePath},
};

use anyhow::{anyhow, Result};

const WALLSHIFT_DIR: &str = ".local/share/wallshift";

/// Returns the path to the current wallpaper information file
fn get_wallpaper_info_path() -> Result<String> {
    Ok(format!(
        "{}/{WALLSHIFT_DIR}/.current_wallpaper",
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

/// Gets the current wallpaper that has been stored on a particular config file.
pub fn get_current_wallpaper() -> Result<File> {
    let wallpaper_info_path = get_wallpaper_info_path()?;

    let wallpaper = read_to_string(wallpaper_info_path)
        .map_err(|_| anyhow!("failed to open the wallpaper directory, it appears to be missing"))?;

    File::try_from(wallpaper).map_err(|msg| anyhow!("failed to get current wallpaper: {msg}"))
}

fn get_random_file(files: Vec<&DirEntry>) -> PathBuf {
    let random_number = rand::rng().random_range(0..files.len());
    files.get(random_number).unwrap().path()
}

/// Gets a random wallpaper from the wallpaper directory.
/// It can also return a folder, which will be handled by the caller.
/// Hidden files will be ignored.
pub fn get_random_wallpaper(settings: &Settings) -> Result<File> {
    let files = read_dir(settings.wallpaper_dir.clone())?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                if !entry
                    .file_name()
                    .to_str()
                    .expect("failed to convert file name to str")
                    .starts_with('.')
                {
                    Some(entry)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        return Err(anyhow!("no wallpapers in the wallpaper directory"));
    }

    let path = if let Ok(current_wallpaper) = get_current_wallpaper() {
        let current_wallpaper_str = current_wallpaper.to_string();
        let files = files
            .iter()
            .filter(|entry| {
                let entry_path_str = entry
                    .path()
                    .to_str()
                    .expect("failed to convert path to str")
                    .to_string();
                entry_path_str != current_wallpaper_str
            })
            .collect::<Vec<_>>();
        get_random_file(files)
    } else {
        get_random_file(files.iter().collect())
    };

    File::new(path).ok_or(anyhow!("failed to get random wallpaper"))
}

/// Returns a path to the next animated wallpaper.
/// If the path is a folder it will return the first wallpaper in the folder.
/// If the path is a file it will return the next wallpaper in the folder.
/// If the path is the last wallpaper in the folder it will return None.
pub fn get_next_animated_wallpaper(settings: &Settings, path: &File) -> Result<Option<ImagePath>> {
    let name = path.get_animated_wallpaper_name();
    let next_index;
    match path {
        File::Image(img) => {
            let max_index = read_dir(format!("{}/{name}", settings.wallpaper_dir))?.count();

            // Get the last numbers of the name
            let last_numbers = img.get_animated_number().ok_or(anyhow!(
                "failed to get last numbers of animated wallpaper name"
            ))?;

            next_index = last_numbers + 1;
            if next_index > max_index as u32 {
                return Ok(None);
            }
        }
        File::Folder(_) => {
            next_index = 1;
        }
    }

    //TODO: Add support for other file formats
    Ok(Some(ImagePath::from(format!(
        "{}/{name}/{name}{}.png",
        settings.wallpaper_dir, next_index
    ))))
}

/// Gets the next wallpaper.
pub fn get_next_wallpaper(settings: &Settings) -> Result<ImagePath> {
    let mut current_wallpaper = get_current_wallpaper().unwrap_or(get_random_wallpaper(settings)?);
    let mut new_wallpaper = get_random_wallpaper(settings)?;
    if current_wallpaper.is_animated(settings) {
        update_animated(settings, &current_wallpaper)
    } else if new_wallpaper.is_animated(settings) {
        update_animated(settings, &new_wallpaper)
    } else {
        match new_wallpaper {
            File::Image(img) => Ok(img),
            File::Folder(_) => unreachable!(),
        }
    }
}

pub fn update_animated(settings: &Settings, path: &File) -> Result<ImagePath> {
    let next_wallpaper = get_next_animated_wallpaper(settings, path)?;
    if let Some(next_wallpaper) = next_wallpaper {
        Ok(next_wallpaper)
    } else {
        let mut new_random = get_random_wallpaper(settings)?;
        if new_random.is_animated(settings) {
            update_animated(settings, &new_random)
        } else {
            match new_random {
                File::Image(img) => Ok(img),
                File::Folder(_) => unreachable!(),
            }
        }
    }
}

/// Updates the current wallpaper using feh.
/// If the option is selected it will also update the betterlockscreen wallpaper.
pub fn update_wallpaper(settings: &Settings, path: &str) -> Result<()> {
    // TODO: allow user to choose other wallpaper setter
    Command::new("feh").arg("--bg-fill").arg(path).output()?;

    // Updates the betterlockscreen wallpaper
    if settings.betterlockscreen {
        Command::new("betterlockscreen")
            .arg("-u")
            .arg(path)
            .output()?;
    }

    // Saves the current wallpaper
    save_wallpaper(path)?;

    Ok(())
}

/// Saves the path to the current wallpaper on the right file
fn save_wallpaper(path: &str) -> Result<()> {
    std::fs::create_dir_all(format!(
        "{}/
        WALLSHIFT_DIR",
        get_home_dir()?
    ))?;

    std::fs::write(get_wallpaper_info_path()?, path)?;

    Ok(())
}
