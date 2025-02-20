use rand::Rng;
use std::{
    fs::{read_dir, read_to_string},
    path::PathBuf,
    process::Command,
};

use crate::{
    configuration::Settings,
    error::{ExecError, FileError, ParsingError, WallshiftError},
    path::{File, ImagePath},
};

fn get_home_dir() -> Result<String, WallshiftError> {
    let home = home::home_dir()
        .ok_or(FileError {
            message: "failed to get home directory".to_owned(),
        })?
        .to_str()
        .ok_or(ParsingError {
            message: "failed to convert home directory to str".to_owned(),
        })?
        .to_owned();
    Ok(home)
}

/// Gets the current wallpaper that has been stored on a particular config file.
pub fn get_current_wallpaper() -> Result<File, WallshiftError> {
    let wallpaper_path = read_to_string(format!(
        "{}/.local/share/wallshift/.current_wallpaper",
        get_home_dir()?
    ))
    .map_err(|msg| FileError {
        message: format!("failed to get current wallpaper: {msg}"),
    })?;

    File::try_from(wallpaper_path).map_err(|msg| {
        FileError {
            message: format!("failed to get current wallpaper: {msg}"),
        }
        .into()
    })
}

/// Gets a random wallpaper from the wallpaper directory.
/// It can also return a folder, which will be handled by the caller.
/// Hidden files will be ignored.
pub fn get_random_wallpaper(settings: &Settings) -> Result<File, WallshiftError> {
    let files = read_dir(settings.wallpaper_dir.clone())
        .map_err(|_| {
            Into::<WallshiftError>::into(FileError {
                message: "failed to open the wallpaper directory, it appears to be missing"
                    .to_owned(),
            })
        })?
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
        return Err(FileError {
            message: "no wallpapers in the wallpaper directory".to_owned(),
        }
        .into());
    }

    let get_path_str = |path: &PathBuf| -> Result<String, WallshiftError> {
        path.to_str()
            .ok_or(
                ParsingError {
                    message: "failed to convert path to str".to_owned(),
                }
                .into(),
            )
            .map(|path_str| path_str.to_owned())
    };

    let path = if let Ok(current_wallpaper) = get_current_wallpaper() {
        let current_wallpaper_str = current_wallpaper.to_string();
        let files = files
            .iter()
            .filter(|entry| {
                let entry_path = entry.path();
                let entry_path_str = get_path_str(&entry_path).unwrap();
                entry_path_str != current_wallpaper_str
            })
            .collect::<Vec<_>>();

        let random_number = rand::rng().random_range(0..files.len());
        files.get(random_number).unwrap().path()
    } else {
        let random_number = rand::rng().random_range(0..files.len());
        files.get(random_number).unwrap().path()
    };

    File::new(path).ok_or(
        FileError {
            message: "failed to get random wallpaper".to_owned(),
        }
        .into(),
    )
}

/// Returns a path to the next animated wallpaper.
/// If the path is a folder it will return the first wallpaper in the folder.
/// If the path is a file it will return the next wallpaper in the folder.
/// If the path is the last wallpaper in the folder it will return None.
pub fn get_next_animated_wallpaper(
    settings: &Settings,
    path: &File,
) -> Result<Option<ImagePath>, WallshiftError> {
    let name = path.get_animated_wallpaper_name();
    let next_index;
    match path {
        File::Image(img) => {
            let max_index = read_dir(format!("{}/{name}", settings.wallpaper_dir))
                .map_err(|_|{
                    Into::<WallshiftError>::into(FileError {
                    message: "failed to open the animated wallpaper directory, it appears to be missing".to_owned(),
                })})?
                .count();

            // Get the last numbers of the name
            let last_numbers = img.get_animated_number().ok_or(ParsingError {
                message: "failed to get last numbers of animated wallpaper name".to_owned(),
            })?;
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
pub fn get_next_wallpaper(settings: &Settings) -> Result<ImagePath, WallshiftError> {
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

pub fn update_animated(settings: &Settings, path: &File) -> Result<ImagePath, WallshiftError> {
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
/// If the option is selected it wull also update the betterlockscreen wallpaper.
///
/// # Panics
///
/// Panics if the call to feh fails.
/// Panics if the call to betterlockscreen fails.
///
pub fn update_wallpaper(settings: &Settings, path: &str) -> Result<(), WallshiftError> {
    // TODO: allow user to choose other wallpaper setter
    Command::new("feh")
        .arg("--bg-fill")
        .arg(path)
        .output()
        .map_err(|err| {
            Into::<WallshiftError>::into(ExecError {
                message: format!("failed to update wallpaper: {err}"),
            })
        })?;

    // Updates the betterlockscreen wallpaper
    if settings.betterlockscreen {
        Command::new("betterlockscreen")
            .arg("-u")
            .arg(path)
            .output()
            .map_err(|err| {
                Into::<WallshiftError>::into(ExecError {
                    message: format!("failed to update betterlockscreen wallpaper: {err}"),
                })
            })?;
    }

    // Saves the current wallpaper

    let home = get_home_dir()?;

    std::fs::create_dir_all(format!("{home}/.local/share/wallshift")).map_err(|err| {
        Into::<WallshiftError>::into(FileError {
            message: format!("failed to create wallshift directory: {err}"),
        })
    })?;

    std::fs::write(
        format!("{home}/.local/share/wallshift/.current_wallpaper",),
        path,
    )
    .map_err(|err| {
        Into::<WallshiftError>::into(FileError {
            message: format!("failed to write current wallpaper: {err}"),
        })
    })?;

    Ok(())
}
