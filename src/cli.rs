use clap_derive::Parser;

use crate::configuration::{get_configuration, Settings};

pub enum Actions {
    Launch,
    Toggle,
    Get,
    Stop,
    Resume,
    Set(String),
}

#[derive(Parser)]
pub struct Cli {
    /// Toggle wallpaper
    #[clap(short, long, conflicts_with_all = &["get", "seconds", "minutes"])]
    toggle: bool,

    /// Time between toggles in seconds. If not it defaults to 1800 seconds
    #[clap(short, long, group = "sleep", conflicts_with = "minutes")]
    seconds: Option<u64>,

    /// Time between toggles in minutes. If not it defaults to 30 minutes
    #[clap(short, long, group = "sleep", conflicts_with = "seconds")]
    minutes: Option<u64>,

    /// Get current wallpaper
    #[clap(short, long, conflicts_with_all = &["toggle", "seconds", "minutes", "betterlockscreen"])]
    get: bool,

    /// Sets the current wallpaper to a specific one.
    #[clap(long, conflicts_with_all = &["toggle", "seconds", "minutes", "betterlockscreen", "reset", "get"])]
    set: Option<String>,

    /// Resumes the usual cycle
    #[clap(long, conflicts_with_all = &["toggle", "seconds", "minutes", "betterlockscreen", "reset", "get", "stop"])]
    resume: bool,

    /// Stops the slideshow. you can resume with `--resume`
    #[clap(long, conflicts_with_all = &["toggle", "seconds", "minutes", "betterlockscreen", "reset", "get", "resume"])]
    stop: bool,

    /// Updates the betterlockscreen wallpaper
    #[clap(long, group = "input")]
    betterlockscreen: Option<bool>,
}

impl Cli {
    const fn get_seconds(&self) -> Option<u64> {
        if let Some(seconds) = self.seconds {
            return Some(seconds);
        }
        if let Some(minutes) = self.minutes {
            return Some(minutes * 60);
        }
        None
    }

    #[must_use]
    pub fn get_settings(&self) -> Settings {
        let mut settings = get_configuration().unwrap_or_else(|_| Settings::default());

        if let Some(seconds) = self.get_seconds() {
            settings.sleep_time = seconds;
        }

        if let Some(betterlockscreen) = self.betterlockscreen {
            settings.betterlockscreen = betterlockscreen;
        }

        settings
    }

    #[must_use]
    pub fn get_action(&self) -> Actions {
        if self.toggle {
            return Actions::Toggle;
        }
        if self.get {
            return Actions::Get;
        }
        if self.resume {
            return Actions::Resume;
        }

        if self.stop {
            return Actions::Stop;
        }
        if let Some(wal) = self.set.clone() {
            return Actions::Set(wal);
        }

        Actions::Launch
    }
}
