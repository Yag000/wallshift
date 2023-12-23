use std::{fs::File, thread, time::Duration};

use daemonize::Daemonize;

use crate::{
    cli::Actions,
    configuration::Settings,
    wallpaper::{get_next_wallpaper, update_wallpaper},
};

pub fn run(settings: Settings, action: Actions) {
    match action {
        Actions::Launch => run_daemon(settings),
        Actions::Toggle => {
            let wallpaper = get_next_wallpaper(&settings);
            let path = wallpaper.to_string();
            if let Err(err) = update_wallpaper(&settings, &path) {
                eprintln!("Error, {}", err);
            }
        }
        Actions::Get => {
            let wallpaper = get_next_wallpaper(&settings);
            println!("{}", wallpaper.to_string());
        }
    }
}

fn run_daemon(settings: Settings) {
    let stdout = File::create("/tmp/wallshift.out").unwrap();
    let stderr = File::create("/tmp/wallshift.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/wallshift.pid")
        .chown_pid_file(true)
        .stdout(stdout) // Redirect stdout
        .stderr(stderr); // Redirect stderr

    match daemonize.start() {
        Ok(_) => launch_wallpaper_loop(settings),
        Err(e) => eprintln!("Error, {}", e),
    }
}

fn launch_wallpaper_loop(settings: Settings) {
    loop {
        let mut wallpaper = get_next_wallpaper(&settings);
        let path = wallpaper.to_string();
        if let Err(err) = update_wallpaper(&settings, &path) {
            eprintln!("Error, {}", err);
        }

        let sleep_time = wallpaper.get_sleep_time(&settings);
        thread::sleep(Duration::from_secs(sleep_time));
    }
}
