
#[macro_use]
extern crate log;

use {
    broot::cli,
    log::LevelFilter,
    simplelog,
    std::{
        env,
        fs::File,
        str::FromStr,
    },
};

/// configure the application log according to env variable.
///
/// There's no log unless the BROOT_LOG environment variable is set to
///  a valid log level (trace, debug, info, warn, error, off)
/// Example:
///      BROOT_LOG=info broot
/// As broot is a terminal application, we only log to a file (dev.log)
fn configure_log() {
    let level = env::var("BROOT_LOG").unwrap_or_else(|_| "off".to_string());
    if level == "off" {
        return;
    }
    if let Ok(level) = LevelFilter::from_str(&level) {
        simplelog::WriteLogger::init(
            level,
            simplelog::Config::default(),
            File::create("dev.log").expect("Log file can't be created"),
        )
        .expect("log initialization failed");
        info!(
            "Starting Broot v{} with log level {}",
            env!("CARGO_PKG_VERSION"),
            level
        );
    }
}

fn main() {
    configure_log();
    match cli::run() {
        Ok(Some(launchable)) => {
            if let Err(e) = launchable.execute() {
                warn!("Failed to launch {:?}", &launchable);
                warn!("Error: {:?}", e);
                eprintln!("{}", e);
            }
        }
        Ok(None) => {}
        Err(e) => {
            // this usually happens when the passed path isn't of a directory
            warn!("Error: {}", e);
            eprintln!("{}", e);
        }
    };
    info!("bye");
}
