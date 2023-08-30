use std::process::Command;

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
pub fn get_current_wallpaper() -> String {
    let feh_raw = std::fs::read_to_string("~/.fehbg").expect("failed to open .fehbg file");

    feh_raw
        .lines()
        .nth(1)
        .expect("failed to parse .fehbg file, it should contain at least 2 lines")
        .split(' ')
        .last()
        .expect(
            "failed to parse .fehbg file, the last line should contain the path to the wallpaper",
        )
        .to_string()
}

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
