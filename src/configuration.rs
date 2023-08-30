#[derive(serde::Deserialize)]
pub struct Settings {
    pub wallpaper_folder: String,
    pub betterlockscreen: bool,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // TODO: Add more possible sources
    // in particular $HOME/.config/wallpaper-updaterrc.yml
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "wallpaper-updaterrc.yml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
