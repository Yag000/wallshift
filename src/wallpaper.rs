use rand::Rng;
use std::{
    fs::{read_dir, read_to_string},
    process::Command,
};

use crate::{
    configuration::Settings,
    error::*,
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
///
/// # Panics
///
/// Panics if the wallpaper directory does not exist.
///
pub fn get_random_wallpaper(settings: &Settings) -> Result<File, WallshiftError> {
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

    let files_len = files.len();

    if files_len == 0 {
        return Err(FileError {
            message: "no wallpapers in the wallpaper directory".to_owned(),
        }
        .into());
    }

    let mut random_number = rand::thread_rng().gen_range(0..files.len());

    let mut path = files.get(random_number).unwrap().as_ref().unwrap().path();

    if let Ok(current_wallpaper) = get_current_wallpaper() {
        let current_wallpaper = current_wallpaper.to_string();
        let path_str = path
            .to_str()
            .ok_or(ParsingError {
                message: "failed to convert path to str".to_owned(),
            })?
            .to_owned();

        while path_str == current_wallpaper && files_len > 1 {
            random_number = rand::thread_rng().gen_range(0..files.len());
            path = files.get(random_number).unwrap().as_ref().unwrap().path();
        }
    }

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
pub fn update_wallpaper(settings: &Settings, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: allow user to choose other wallpaper setter
    Command::new("feh").arg("--bg-fill").arg(path).output()?;

    // Updates the betterlockscreen wallpaper
    if settings.betterlockscreen {
        Command::new("betterlockscreen")
            .arg("-u")
            .arg(path)
            .output()
            .expect("failed to call betterlockscreen");
    }

    // Saves the current wallpaper
    //
    let home = home::home_dir()
        .ok_or("failed to get home directory")?
        .to_str()
        .ok_or("failed to convert home directory to str")?
        .to_owned();

    std::fs::create_dir_all(format!("{home}/.local/share/wallshift"))?;

    std::fs::write(
        format!("{home}/.local/share/wallshift/.current_wallpaper",),
        path,
    )?;

    Ok(())
}
