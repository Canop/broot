use {
    crate::kitty::KittyGraphicsDisplay,
    cli_log::*,
    std::{
        cmp::Ordering,
        env,
    },
};

/// Whether the WezTerm build identified by `$TERM_PROGRAM_VERSION`
/// supports the Kitty Graphics protocol.
///
/// A missing version is assumed to be a recent, supporting build.
fn wezterm_supports_kitty_graphics(version: Option<&str>) -> bool {
    match version {
        // a WezTerm build id is a fixed-width `YYYYMMDD-HHMMSS-hash`, for which
        // a lexicographic compare is correct — do NOT use compare_versions here
        Some(version) => version >= "20220105-201556-91a423da",
        None => true,
    }
}

/// Compare two dotted version strings numerically, component by component, so
/// `"3.6.10" > "3.6.6"` (unlike a lexicographic `str` comparison).
///
/// An optional leading `v`/`V` is ignored (`"v1.3.2"`), and any pre-release or
/// build suffix introduced by `-` or `+` is dropped before comparing, so
/// `"1.2.3-rc.1"` and `"1.2.3+build.7"` both compare as `"1.2.3"`. Within the
/// numeric core each `.`-separated component is compared by its integer value
/// and missing trailing components count as 0 (`"3.6" == "3.6.0"`).
///
/// This does NOT implement semver pre-release precedence (an `-rc` is treated as
/// its base release); it's for simple "is it recent enough" gates.
fn compare_versions(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (version_core(a).split('.'), version_core(b).split('.'));
    loop {
        match (a.next(), b.next()) {
            (None, None) => return Ordering::Equal,
            (x, y) => {
                let ord = component(x.unwrap_or("")).cmp(&component(y.unwrap_or("")));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
}

/// The numeric `MAJOR.MINOR.PATCH…` core of a version string: strip an optional
/// leading `v`/`V`, then cut at the first character that isn't a digit or `.`
/// (dropping any `-pre` / `+build` suffix).
fn version_core(s: &str) -> &str {
    let s = s.strip_prefix(['v', 'V']).unwrap_or(s);
    let end = s.find(|c: char| c != '.' && !c.is_ascii_digit()).unwrap_or(s.len());
    &s[..end]
}

/// A single `.`-separated version component as a `u64` (0 if empty / unparsable).
fn component(s: &str) -> u64 {
    s.parse().unwrap_or(0)
}

/// Whether dotted version `version` is at least `min` (numeric; see `compare_versions`).
fn version_at_least(version: &str, min: &str) -> bool {
    compare_versions(version, min) != Ordering::Less
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

    // we detect rio by the $TERM_PROGRAM env var
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        debug!("$TERM_PROGRAM = {term_program:?}");
        if term_program == "rio" {
            debug!(" -> this terminal seems to be rio");
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

                if version_at_least(&version, "3.6.6") {
                    debug!("this looks like a compatible version");
                    return KittyGraphicsDisplay::Direct;
                } else {
                    debug!("iTerm2's version predates Kitty Graphics protocol support");
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
    use {
        super::{compare_versions, version_at_least, wezterm_supports_kitty_graphics},
        std::cmp::Ordering,
    };

    #[test]
    fn compares_versions_numerically_not_lexically() {
        // the lexicographic bug this fixes: "3.6.10" must be > "3.6.6"
        assert_eq!(compare_versions("3.6.10", "3.6.6"), Ordering::Greater);
        assert_eq!(compare_versions("3.6.6", "3.6.6"), Ordering::Equal);
        assert_eq!(compare_versions("3.6", "3.6.0"), Ordering::Equal); // missing component = 0
        assert_eq!(compare_versions("3.10.0", "3.9.9"), Ordering::Greater);
        assert_eq!(compare_versions("2.9.9", "3.0.0"), Ordering::Less);
        assert_eq!(compare_versions("3.6.0-nightly", "3.6.0"), Ordering::Equal); // suffix ignored
    }

    #[test]
    fn compare_versions_handles_prefix_and_dotted_suffix() {
        // leading v/V is stripped
        assert_eq!(compare_versions("v1.3.2", "1.3.2"), Ordering::Equal);
        assert_eq!(compare_versions("v1.3.10", "V1.3.2"), Ordering::Greater);
        // a dotted pre-release / build suffix is dropped (compared as base release),
        // not turned into a spurious extra component
        assert_eq!(compare_versions("1.2.3-rc.1", "1.2.3"), Ordering::Equal);
        assert_eq!(compare_versions("1.2.3+build.7", "1.2.3"), Ordering::Equal);
        assert_eq!(compare_versions("3.6.20250808-nightly", "3.6.6"), Ordering::Greater);
    }

    #[test]
    fn version_at_least_gates_correctly() {
        assert!(version_at_least("3.6.6", "3.6.6"));
        assert!(version_at_least("3.6.10", "3.6.6")); // string compare got this wrong
        assert!(version_at_least("3.7.0", "3.6.6"));
        assert!(!version_at_least("3.6.5", "3.6.6"));
        assert!(!version_at_least("2.9.9", "3.6.6"));
    }

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
