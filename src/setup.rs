use crate::{
    configuration::Settings,
    wallpaper::{get_next_wallpaper, update_wallpaper},
};

pub fn run(settings: Settings) {
    let next_wallpaper = get_next_wallpaper(&settings);
    update_wallpaper(&settings, next_wallpaper.to_str().unwrap());
}
