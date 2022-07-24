use {
    cli_log::*,
    std::{
        env,
    },
};

/// Determine whether Kitty's graphics protocol is supported
/// by the terminal running broot.
///
/// This is called only once, and cached in the KittyManager's
/// MaybeRenderer state
#[allow(unreachable_code)]
pub fn is_kitty_graphics_protocol_supported() -> bool {
    debug!("is_kitty_graphics_protocol_supported ?");

    #[cfg(not(unix))]
    {
        // because cell_size_in_pixels isn't implemented on Windows
        debug!("no kitty support yet on Windows");
        return false;
    }

    // we detect Kitty by the $TERM or $TERMINAL env var
    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            debug!("${} = {:?}", env_var, env_val);
            let env_val = env_val.to_ascii_lowercase();
            if env_val.contains("kitty") {
                debug!(" -> this terminal seems to be Kitty");
                return true;
            }
        }
    }

    // we detect Wezterm with the $TERM_PROGRAM env var and we
    // check its version to be sure it's one with support
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        debug!("$TERM_PROGRAM = {:?}", term_program);
        if term_program == "WezTerm" {
            if let Ok(version) = env::var("TERM_PROGRAM_VERSION") {
                debug!("$TERM_PROGRAM_VERSION = {:?}", version);
                if &*version < "20220105-201556-91a423da" {
                    debug!("WezTerm's version predates Kitty Graphics protocol support");
                } else {
                    debug!("this looks like a compatible version");
                    return true;
                }
            } else {
                warn!("$TERM_PROGRAM_VERSION unexpectedly missing");
            }
        }
    }

    // Checking support with a proper CSI sequence should be the preferred way but
    // it doesn't work reliably on wezterm and requires a wait on other terminals.
    // As both Kitty and WezTerm set env vars allowing an easy detection, this
    // CSI based querying isn't necessary right now.
    // This feature is kept gated and should only be tried if other terminals
    // appear and can't be detected without CSI sequence.
    #[cfg(feature = "kitty-csi-check")]
    {
        let start = std::time::Instant::now();
        const TIMEOUT_MS: isize = 400;
        let s = match xterm_query::query("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c", TIMEOUT_MS) {
            Err(e) => {
                debug!("xterm querying failed: {}", e);
                false
            }
            Ok(response) => {
                response.starts_with("\x1b_Gi=31;OK\x1b")
            }
        };
        debug!("Xterm querying took {:?}", start.elapsed());
        debug!("kitty protocol support: {:?}", s);
        return s;
    }
    false
}

