use clap::Parser;

use wallpaper_updater::cli::Cli;
use wallpaper_updater::setup::run;

fn main() {
    let cli = Cli::parse();
    let config = cli.get_settings();
    let action = cli.get_action();
    run(config, action);
}
