use clap::Parser;

use wallpaper_updater::cli::Cli;
use wallpaper_updater::setup::run;

fn main() {
    let config = Cli::parse().get_settings();
    run(config);
}
