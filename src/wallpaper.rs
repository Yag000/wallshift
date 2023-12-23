use rand::Rng;
use std::{
    fs::{read_dir, read_to_string},
    process::Command,
};

use crate::{
    configuration::Settings,
    path::{File, ImagePath},
};

/// Gets the current wallpaper. It assumes that the user is only using feh to set the wallpaper.
/// TODO: allow user to choose other wallpaper setter
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
pub fn get_current_wallpaper() -> Option<File> {
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

    File::try_from(wallpaper_path).ok()
}

/// Gets a random wallpaper from the wallpaper directory.
/// It can also return a folder, which will be handled by the caller.
/// Hidden files will be ignored.
///
/// # Panics
///
/// Panics if the wallpaper directory does not exist.
///
pub fn get_random_wallpaper(settings: &Settings) -> Option<File> {
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

    let mut random_number = rand::thread_rng().gen_range(0..files.len());

    let mut path = files.get(random_number).unwrap().as_ref().unwrap().path();

    if let Some(current_wallpaper) = get_current_wallpaper() {
        let current_wallpaper = current_wallpaper.to_string();
        while *path.to_str()? == current_wallpaper && files.len() > 1 {
            random_number = rand::thread_rng().gen_range(0..files.len());
            path = files.get(random_number).unwrap().as_ref().unwrap().path();
        }
    }

    File::new(path)
}

/// Gets a random wallpaper from the wallpaper directory.
/// It will not return a folder.
/// Hidden files will be ignored.
///
/// # Panics
///
/// Panics if the wallpaper directory does not exist.
///
pub fn get_random_wallpaper_file(settings: &Settings) -> Option<ImagePath> {
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

    let path = files.get(random_number).unwrap().as_ref().unwrap().path();
    ImagePath::new(path)
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
pub fn get_next_animated_wallpaper(settings: &Settings, path: &File) -> Option<ImagePath> {
    let name = path.get_animated_wallpaper_name();
    let next_index;
    match path {
        File::Image(img) => {
            let max_index = read_dir(format!("{}/{name}", settings.wallpaper_dir))
                .expect("failed to open wallpaper directory")
                .count();

            // Get the last numbers of the name
            let last_numbers = img.get_animated_number().unwrap();
            next_index = last_numbers + 1;
            if next_index > max_index as u32 {
                return None;
            }
        }
        File::Folder(_) => {
            next_index = 1;
        }
    }

    //TODO: Add support for other file formats
    Some(ImagePath::from(format!(
        "{}/{name}/{name}{}.png",
        settings.wallpaper_dir, next_index
    )))
}

/// Gets the next wallpaper.
pub fn get_next_wallpaper(settings: &Settings) -> ImagePath {
    let mut current_wallpaper =
        get_current_wallpaper().unwrap_or(get_random_wallpaper(settings).unwrap());
    let mut new_wallpaper = get_random_wallpaper(settings)
        .expect("failed to get random wallpaper, not enough wallpapers in the wallpaper directory");
    if current_wallpaper.is_animated(settings) {
        update_animated(settings, &current_wallpaper)
    } else if new_wallpaper.is_animated(settings) {
        update_animated(settings, &new_wallpaper)
    } else {
        match new_wallpaper {
            File::Image(img) => img,
            File::Folder(_) => unreachable!(),
        }
    }
}

pub fn update_animated(settings: &Settings, path: &File) -> ImagePath {
    let next_wallpaper = get_next_animated_wallpaper(settings, path);
    if let Some(next_wallpaper) = next_wallpaper {
        next_wallpaper
    } else {
        let mut new_random = get_random_wallpaper(settings).expect(
            "failed to get random wallpaper, not enough wallpapers in the wallpaper directory",
        );
        if new_random.is_animated(settings) {
            update_animated(settings, &new_random)
        } else {
            match new_random {
                File::Image(img) => img,
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
