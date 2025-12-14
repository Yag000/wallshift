use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub wallpaper_dir: String,
    pub betterlockscreen: bool,
    pub sleep_time: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            wallpaper_dir: format!(
                "{}/Pictures/Wallpapers",
                dirs::home_dir().unwrap().to_str().unwrap()
            ),
            betterlockscreen: false,
            sleep_time: 1800,
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let config_path = format!(
        "{}/wallshift/config.yml",
        dirs::config_dir().unwrap().to_str().unwrap()
    );
    let settings = config::Config::builder()
        .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
        .build()?;

    settings.try_deserialize::<Settings>()
}
