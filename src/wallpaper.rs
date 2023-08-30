use std::{
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    process::Command,
};

use rand::Rng;

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
    let feh_raw = read_to_string("~/.fehbg").expect("failed to open .fehbg file");

    let wallpaper_path = feh_raw
        .lines()
        .nth(1)
        .expect("failed to parse .fehbg file, it should contain at least 2 lines")
        .split(' ')
        .last()
        .expect(
            "failed to parse .fehbg file, the last line should contain the path to the wallpaper",
        )
        .to_string();

    PathBuf::from(wallpaper_path)
}

/// Gets a random wallpaper from the wallpaper directory.
/// It can also return a folder, which will be handled by the caller.
///
/// # Panics
///
/// Panics if the wallpaper directory does not exist.
///
pub fn get_random_wallpaper(settings: &Settings) -> Option<PathBuf> {
    let files = read_dir(settings.wallpaper_dir.clone())
        .expect("failed to open wallpaper directory")
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

/// Gets a random wallpaper from the wallpaper directory.
/// It will not return a folder.
///
/// # Panics
///
/// Panics if the wallpaper directory does not exist.
///
pub fn get_random_wallpaper_file(settings: &Settings) -> Option<PathBuf> {
    let files = read_dir(settings.wallpaper_dir.clone())
        .expect("failed to open wallpaper directory")
        // Filter out folders
        .filter(|entry| entry.as_ref().unwrap().path().is_file())
        .collect::<Vec<_>>();

    if files.is_empty() {
        return None;
    }

    let random_number = rand::thread_rng().gen_range(0..files.len());
    Some(files.get(random_number).unwrap().as_ref().unwrap().path())
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
