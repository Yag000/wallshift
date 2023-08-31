use clap::Parser;

use wallshift::cli::Cli;
use wallshift::setup::run;

fn main() {
    let cli = Cli::parse();
    let config = cli.get_settings();
    let action = cli.get_action();
    run(config, action);
}
