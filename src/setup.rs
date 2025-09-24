use std::{fs::File, thread, time::Duration};

use daemonize::Daemonize;

use crate::{
    cli::Actions,
    configuration::Settings,
    data::{set_off, set_on},
    wallpaper::{get_next_wallpaper, update_wallpaper},
};

fn toggle(settings: &Settings) {
    match get_next_wallpaper(settings) {
        Ok(wallpaper) => {
            let path = wallpaper.to_string();
            if let Err(err) = update_wallpaper(settings, &path) {
                eprintln!("Error, {err}");
            }
        }
        Err(err) => eprintln!("Error, {err}"),
    }
}

pub fn run(settings: Settings, action: Actions) {
    match action {
        Actions::Launch => run_daemon(settings),
        Actions::Toggle => toggle(&settings),
        Actions::Get => match get_next_wallpaper(&settings) {
            Ok(wallpaper) => println!("{wallpaper}"),
            Err(err) => eprintln!("Error, {err}"),
        },
        Actions::Resume => match set_on() {
            Ok(()) => (),
            Err(err) => eprintln!("Error, {err}"),
        },
        Actions::Stop => match set_off() {
            Ok(()) => (),
            Err(err) => eprintln!("Error, {err}"),
        },
        Actions::Set(wall) => match update_wallpaper(&settings, &wall) {
            Ok(()) => (),
            Err(err) => eprintln!("Error, {err}"),
        },
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
        Ok(()) => launch_wallpaper_loop(settings),
        Err(e) => eprintln!("Error, {e}"),
    }
}

fn launch_wallpaper_loop(settings: Settings) {
    loop {
        match get_next_wallpaper(&settings) {
            Ok(mut wallpaper) => {
                let path = wallpaper.to_string();
                if let Err(err) = update_wallpaper(&settings, &path) {
                    eprintln!("Error, {err}");
                    thread::sleep(Duration::from_secs(settings.sleep_time));
                } else {
                    let sleep_time = match wallpaper.get_sleep_time(&settings) {
                        Ok(seconds) => seconds,
                        Err(err) => {
                            eprintln!("Error, {err}");
                            settings.sleep_time
                        }
                    };

                    thread::sleep(Duration::from_secs(sleep_time));
                }
            }
            Err(err) => eprintln!("Error, {err}"),
        }
    }
}
