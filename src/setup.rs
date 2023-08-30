use crate::{
    cli::Actions,
    configuration::Settings,
    wallpaper::{get_next_wallpaper, update_wallpaper},
};

pub fn run(settings: Settings, action: Actions) {
    match action {
        Actions::Launch => {
            todo!();
        }
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
