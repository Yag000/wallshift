use std::{thread, time::Duration};

use crate::{
    cli::Actions,
    configuration::Settings,
    wallpaper::{get_next_wallpaper, get_sleep_time, update_wallpaper},
};

pub fn run(settings: Settings, action: Actions) {
    match action {
        Actions::Launch => loop {
            let wallpaper = get_next_wallpaper(&settings);
            let path = wallpaper.to_str().unwrap();
            update_wallpaper(&settings, path);
            let sleep_time = get_sleep_time(&settings, &wallpaper);
            thread::sleep(Duration::from_secs(sleep_time));
        },
        Actions::Toggle => {
            let wallpaper = get_next_wallpaper(&settings);
            let path = wallpaper.to_str().unwrap();
            update_wallpaper(&settings, path);
        }
        Actions::Get => {
            let wallpaper = get_next_wallpaper(&settings);
            println!("{}", wallpaper.to_str().unwrap());
        }
    }
}
