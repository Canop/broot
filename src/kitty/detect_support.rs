use {
    crate::kitty::KittyGraphicsDisplay,
    cli_log::*,
    std::env,
};

/// Whether the WezTerm build identified by `$TERM_PROGRAM_VERSION`
/// supports the Kitty Graphics protocol.
///
/// A missing version is assumed to be a recent, supporting build.
fn wezterm_supports_kitty_graphics(version: Option<&str>) -> bool {
    match version {
        Some(version) => version >= "20220105-201556-91a423da",
        None => true,
    }
}

/// Determine whether Kitty's graphics protocol is supported
/// by the terminal running broot.
///
/// This is called only once, and cached in the `GraphicsManager`'s
/// `MaybeRenderer` state
#[allow(unreachable_code)]
pub fn detect_kitty_graphics_protocol_display() -> KittyGraphicsDisplay {
    debug!("is_kitty_graphics_protocol_supported ?");

    #[cfg(not(unix))]
    {
        // because cell_size_in_pixels isn't implemented on Windows
        debug!("no kitty support yet on Windows");
        return KittyGraphicsDisplay::None;
    }

    // we detect Kitty by the $TERM or $TERMINAL env var
    // check its version to be sure it's one with support
    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            debug!("${env_var} = {env_val:?}");
            let env_val = env_val.to_ascii_lowercase();
            if env_val.contains("kitty") {
                debug!(" -> this terminal seems to be Kitty");
                return KittyGraphicsDisplay::Direct;
            }
        }
    }

    // we detect Ghostty by the $TERM env var
    if let Ok(env_val) = env::var("TERM") {
        debug!("$TERM = {env_val:?}");
        if env_val == "xterm-ghostty" {
            debug!(" -> this terminal seems to be Ghostty");
            return KittyGraphicsDisplay::Direct;
        }
    }

    // we detect Wezterm with the $TERM_PROGRAM env var and we
    // check its version to be sure it's one with support
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        debug!("$TERM_PROGRAM = {term_program:?}");
        if term_program == "WezTerm" {
            let version = env::var("TERM_PROGRAM_VERSION").ok();
            debug!("$TERM_PROGRAM_VERSION = {version:?}");
            if wezterm_supports_kitty_graphics(version.as_deref()) {
                debug!("this looks like a compatible version");
                return KittyGraphicsDisplay::Direct;
            }
            debug!("WezTerm's version predates Kitty Graphics protocol support");
        } else if term_program == "ghostty" {
            debug!("Ghostty implements Kitty Graphics protocol");
            return KittyGraphicsDisplay::Direct;
        } else if term_program == "iTerm.app" {
            if let Ok(version) = env::var("TERM_PROGRAM_VERSION") {
                debug!("$TERM_PROGRAM_VERSION = {version:?}");

                if &*version < "3.6.6" {
                    debug!("iTerm2's version predates Kitty Graphics protocol support");
                } else {
                    debug!("this looks like a compatible version");
                    return KittyGraphicsDisplay::Direct;
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
            Ok(_) => KittyGraphicsDisplay::None,
        };
        debug!("Xterm querying took {:?}", start.elapsed());
        debug!("kitty protocol support: {:?}", s);
        return s;
    }
    KittyGraphicsDisplay::None
}

#[cfg(test)]
mod tests {
    use super::wezterm_supports_kitty_graphics;

    #[test]
    fn wezterm_recent_version_supports_kitty_graphics() {
        assert!(wezterm_supports_kitty_graphics(Some(
            "20230712-072601-f4abf8fd"
        )));
    }

    #[test]
    fn wezterm_threshold_version_supports_kitty_graphics() {
        assert!(wezterm_supports_kitty_graphics(Some(
            "20220105-201556-91a423da"
        )));
    }

    #[test]
    fn wezterm_old_version_does_not_support_kitty_graphics() {
        assert!(!wezterm_supports_kitty_graphics(Some(
            "20210203-095643-70a364eb"
        )));
    }

    #[test]
    fn wezterm_missing_version_is_assumed_supported() {
        assert!(wezterm_supports_kitty_graphics(None));
    }
}
