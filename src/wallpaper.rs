use rand::Rng;
use std::{
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    process::Command,
};

use crate::configuration::Settings;

/// Gets the current wallpaper. It assumes that the user is only using feh to set the wallpaper.
/// TODO: allow user to choose other wallpaper getter
///
///
/// The .fehbg file is a shell script that is run when the user logs in. It is used to set the
/// wallpaper. It uses the following format:
///
/// #!/bin/sh
/// feh --no-fehbg --bg-fill /path/to/wallpaper
///
/// This function parses the .fehbg file and returns the path to the current wallpaper.
///
/// # Panics
///
/// Panics if the file $HOME/.fehbg does not exist.
///
pub fn get_current_wallpaper() -> PathBuf {
    let feh_raw = read_to_string(format!(
        "{}/.fehbg",
        home::home_dir()
            .expect("Unable to get home path")
            .to_str()
            .unwrap()
    ))
    .expect("failed to open .fehbg file");
    let wallpaper_path = feh_raw
        .lines()
        // We take the second line because the first line is the shebang
        .nth(1)
        .expect("failed to parse .fehbg file, it should contain at least 2 lines")
        // We take the last word
        .split(' ')
        .nth(3)
        .expect("failed to parse .fehbg file, it should contain at least 4 words")
        // Remove the single quotes
        .trim_matches('\'')
        .to_string();

    PathBuf::from(wallpaper_path)
}

/// Gets a random wallpaper from the wallpaper directory.
/// It can also return a folder, which will be handled by the caller.
/// Hidden files will be ignored.
///
/// # Panics
///
/// Panics if the wallpaper directory does not exist.
///
pub fn get_random_wallpaper(settings: &Settings) -> Option<PathBuf> {
    let files = read_dir(settings.wallpaper_dir.clone())
        .expect("failed to open wallpaper directory")
        .filter(|entry| {
            !entry
                .as_ref()
                .expect("failed to get entry")
                .file_name()
                .to_str()
                .expect("failed to convert file name to str")
                .starts_with('.')
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        return None;
    }

    let random_number = rand::thread_rng().gen_range(0..files.len());
    Some(files.get(random_number).unwrap().as_ref().unwrap().path())
}

/// Gets a random wallpaper from the wallpaper directory.
/// It will not return a folder.
/// Hidden files will be ignored.
///
/// # Panics
///
/// Panics if the wallpaper directory does not exist.
///
pub fn get_random_wallpaper_file(settings: &Settings) -> Option<PathBuf> {
    let files = read_dir(settings.wallpaper_dir.clone())
        .expect("failed to open wallpaper directory")
        // Filter out folders
        .filter(|entry| {
            let path = entry.as_ref().unwrap().path();
            path.is_file() && !path.as_os_str().to_str().unwrap().starts_with('.')
        })
        .collect::<Vec<_>>();

    if files.is_empty() {
        return None;
    }

    let random_number = rand::thread_rng().gen_range(0..files.len());
    Some(files.get(random_number).unwrap().as_ref().unwrap().path())
}

/// Checks if the given path is an animated wallpaper.\
///
/// An animated wallperper can be either a file or a folder. If it is a folder
/// it must be contained in the wallpaper directory.
/// If it is a file it must be contained in a folder that is contained in the wallpaper directory.
///  
///  Example:
///  wallpaper_dir
///  |--- folder1
///  |    |--- wallpaper1
///  |-- wallpaper2
///
///  folder1 is an animated wallpaper
///  wallpaper1 is an animated wallpaper
///  wallpaper2 is not an animated wallpaper
///
pub fn is_animated_wallpaper(settings: &Settings, path: &Path) -> bool {
    if let Some(parent) = path.parent() {
        if path.is_dir() {
            if let Some(str) = parent.to_str() {
                return str == settings.wallpaper_dir;
            }
            return false;
        } else if let Some(parent) = parent.parent() {
            if let Some(str) = parent.to_str() {
                return str == settings.wallpaper_dir;
            }
        } else {
            return false;
        }
    }
    false
}

/// Gets the name of the folder that contains the given path.
/// If the path is a folder it will return the name of the folder.
///
/// # Panics
///
/// If the path is not an animated wallpaper it may panic.
pub fn get_animated_wallpaper_name(path: &Path) -> String {
    if path.is_dir() {
        path.file_name().unwrap().to_str().unwrap().to_owned()
    } else {
        path.parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }
}

/// Returns a path to the next animated wallpaper.
/// If the path is a folder it will return the first wallpaper in the folder.
/// If the path is a file it will return the next wallpaper in the folder.
/// If the path is the last wallpaper in the folder it will return None.
///
/// # Panics
///
/// Panics if the path is not an animated wallpaper.
/// Panics if the wallpaper directory does not exist.
/// Panics if the path is not a valid path.
/// Panics if the name does not contain a number at the end.
pub fn get_next_animated_wallpaper(settings: &Settings, path: &Path) -> Option<PathBuf> {
    let name = get_animated_wallpaper_name(path);
    let next_index;
    if path.is_dir() {
        next_index = 1;
    } else {
        let max_index = read_dir(format!("{}/{name}", settings.wallpaper_dir))
            .expect("failed to open wallpaper directory")
            .count();

        // Get the last numbers of the name
        let last_numbers = path
            .file_stem()
            .expect("failed to get file name")
            .to_str()
            .expect("failed to convert file name to str")
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();

        next_index = last_numbers.parse::<u32>().unwrap() + 1;
        if next_index > max_index as u32 {
            return None;
        }
    }

    //TODO: Add support for other file formats
    Some(PathBuf::from(format!(
        "{}/{name}/{name}{}.png",
        settings.wallpaper_dir, next_index
    )))
}

/// Gets the next wallpaper.
pub fn get_next_wallpaper(settings: &Settings) -> PathBuf {
    let current_wallpaper = get_current_wallpaper();
    let new_wallpaper = get_random_wallpaper(settings)
        .expect("failed to get random wallpaper, not enough wallpapers in the wallpaper directory");
    if is_animated_wallpaper(settings, &current_wallpaper) {
        update_animated(settings, &current_wallpaper)
    } else if is_animated_wallpaper(settings, &new_wallpaper) {
        update_animated(settings, &new_wallpaper)
    } else {
        new_wallpaper
    }
}

pub fn update_animated(settings: &Settings, path: &Path) -> PathBuf {
    let next_wallpaper = get_next_animated_wallpaper(settings, path);
    if let Some(next_wallpaper) = next_wallpaper {
        next_wallpaper
    } else {
        get_random_wallpaper_file(settings).expect(
            "failed to get random wallpaper, not enough wallpapers in the wallpaper directory",
        )
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
pub fn update_wallpaper(settings: &Settings, path: &str) {
    // TODO: allow user to choose other wallpaper setter
    Command::new("feh")
        .arg("--bg-fill")
        .arg(path)
        .output()
        .expect("failed to call feh");

    // Updates the betterlockscreen wallpaper
    if settings.betterlockscreen {
        Command::new("betterlockscreen")
            .arg("-u")
            .arg(path)
            .output()
            .expect("failed to call betterlockscreen");
    }
}

pub fn get_sleep_time(settings: &Settings, path: &Path) -> u64 {
    if is_animated_wallpaper(settings, path) {
        let number_of_wallpapers = read_dir(path.parent().expect("failed to get parent directory"))
            .expect("failed to open wallpaper directory")
            .count();
        settings.sleep_time / number_of_wallpapers as u64
    } else {
        settings.sleep_time
    }
}
