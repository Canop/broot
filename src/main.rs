
#[macro_use]
extern crate log;

use {
    broot::cli,
};

fn main() {
    cli_log::init("broot");
    match cli::run() {
        Ok(Some(launchable)) => {
            debug!("launching {:#?}", launchable);
            if let Err(e) = launchable.execute(None) {
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
