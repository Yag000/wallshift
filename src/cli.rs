use clap_derive::Parser;

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

    /// Updates the betterlockscreen wallpaper
    #[clap(long, group = "input")]
    betterlockscreen: bool,
}
