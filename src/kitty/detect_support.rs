use {
    crate::kitty::KittyGraphicsDisplay,
    cli_log::*,
    lazy_regex::regex_captures,
    std::{
        env,
        process::Command,
    },
};

/// Determine whether Kitty's graphics protocol is supported
/// by the terminal running broot.
///
/// This is called only once, and cached in the KittyManager's
/// MaybeRenderer state
#[allow(unreachable_code)]
pub fn detect_kitty_graphics_protocol_display() -> KittyGraphicsDisplay {
    debug!("is_kitty_graphics_protocol_supported ?");

    #[cfg(not(unix))]
    {
        // because cell_size_in_pixels isn't implemented on Windows
        debug!("no kitty support yet on Windows");
        return false;
    }

    // we detect Kitty by the $TERM or $TERMINAL env var
    // check its version to be sure it's one with support
    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            debug!("${} = {:?}", env_var, env_val);
            let env_val = env_val.to_ascii_lowercase();
            if env_val.contains("kitty") {
                debug!(" -> this terminal seems to be Kitty");
                return KittyGraphicsDisplay::Direct;
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
                    return KittyGraphicsDisplay::Direct;
                }
            } else {
                warn!("$TERM_PROGRAM_VERSION unexpectedly missing");
            }
        } else if term_program == "ghostty" {
            debug!("Ghostty implements Kitty Graphics protocol");
            return KittyGraphicsDisplay::Direct;
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
        const TIMEOUT_MS: u64 = 200;
        let response = xterm_query::query_osc(
            "\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24;AAAA\x1b\\\x1b[c",
            TIMEOUT_MS,
        );
        let s = match response {
            Err(e) => {
                debug!("xterm querying failed: {}", e);
                KittyGraphicsDisplay::None
            }
            Ok(response) if response == "_Gi=31;OK" => KittyGraphicsDisplay::Direct,
            Ok(response) => KittyGraphicsDisplay::None,
        };
        debug!("Xterm querying took {:?}", start.elapsed());
        debug!("kitty protocol support: {:?}", s);
        return s;
    }
    KittyGraphicsDisplay::None
}

/// Determine whether we're in tmux.
///
/// This is called only once, and cached in KittyImageRenderer
#[allow(unreachable_code)]
pub fn is_tmux() -> bool {
    debug!("is_tmux ?");

    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            debug!("${} = {:?}", env_var, env_val);
            let env_val = env_val.to_ascii_lowercase();
            if env_val.contains("tmux") {
                debug!(" -> this terminal seems to be Tmux");
                return true;
            }
        }
    }
    false
}

/// Custom environment variable to store how deeply tmux is nested. Starts at 1 when there's no nesting.
pub fn get_tmux_nest_count() -> u32 {
    std::env::var("TMUX_NEST_COUNT")
        .map(|s| str::parse(&s).unwrap_or(1))
        .unwrap_or(1)
}

/// Determine whether we're in SSH.
///
/// This is called only once, and cached in KittyImageRenderer
#[allow(unreachable_code)]
pub fn is_ssh() -> bool {
    debug!("is_ssh ?");

    for env_var in ["SSH_CLIENT", "SSH_CONNECTION"] {
        if env::var(env_var).is_ok() {
            debug!(" -> this seems to be under SSH");
            return true;
        }
    }
    false
}
