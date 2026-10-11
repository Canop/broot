use {
    cli_log::*,
    std::process::ExitCode,
};

fn main() -> ExitCode {
    init_cli_log!();
    debug!("env::args(): {:#?}", std::env::args().collect::<Vec<String>>());
    match broot::cli::run() {
        Ok(Some(launchable)) => {
            info!("launching {:#?}", launchable);
            if let Err(e) = launchable.execute(None) {
                warn!("Failed to launch {:?}", launchable);
                warn!("Error: {:?}", e);
                return e.report();
            }
        }
        Ok(None) => {}
        Err(e) => {
            // this usually happens when the passed path isn't of a directory
            warn!("Error: {}", e);
            return e.report();
        }
    };
    log_mem(Level::Info);
    info!("bye");
    ExitCode::SUCCESS
}
