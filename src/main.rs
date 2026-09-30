use cli_log::*;

fn main() {
    init_cli_log!();
    debug!("env::args(): {:#?}", std::env::args().collect::<Vec<String>>());
    match broot::cli::run() {
        Ok(Some(launchable)) => {
            info!("launching {:#?}", launchable);
            if let Err(e) = launchable.execute(None) {
                if e.is_broken_pipe() {
                    std::process::exit(141);
                }
                warn!("Failed to launch {:?}", launchable);
                warn!("Error: {:?}", e);
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        Ok(None) => {}
        Err(e) => {
            if e.is_broken_pipe() {
                std::process::exit(141);
            }
            // this usually happens when the passed path isn't of a directory
            warn!("Error: {}", e);
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    log_mem(Level::Info);
    info!("bye");
}
