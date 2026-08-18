use {
    cli_log::*,
    std::env,
};

/// Timeout for synchronous terminal queries (Sixel DA1, Windows cell-size).
/// Compliant terminals reply within about a millisecond; this is the upper
/// bound before we give up and fall back (to another protocol or no graphics).
pub(crate) const TERMINAL_QUERY_TIMEOUT_MS: u64 = 200;

pub fn get_esc_seq(tmux_nest_count: u32) -> String {
    "\u{1b}".repeat(2usize.pow(tmux_nest_count))
}

pub fn get_tmux_header(tmux_nest_count: u32) -> String {
    let mut header: String = String::new();
    for i in 0..tmux_nest_count {
        header.push_str(&"\u{1b}".repeat(2usize.pow(i)));
        header.push_str("Ptmux;");
    }
    header
}

pub fn get_tmux_tail(tmux_nest_count: u32) -> String {
    let mut tail: String = String::new();
    for i in (0..tmux_nest_count).rev() {
        tail.push_str(&"\u{1b}".repeat(2usize.pow(i)));
        tail.push('\\');
    }
    tail
}

/// Determine whether we're running inside tmux.
///
/// `$TMUX` is set by tmux in every session, whatever `$TERM` is — and `$TERM` is
/// commonly `screen-256color` or `tmux-256color`, so a "does $TERM contain tmux"
/// test misses the frequent `screen*` case. So check `$TMUX` first, and keep the
/// `$TERM`/`$TERMINAL` substring test only as a fallback.
///
/// (We intentionally don't treat a bare `screen*` `$TERM` as tmux: without
/// `$TMUX` that's more likely real GNU screen, whose passthrough differs.)
pub fn is_tmux() -> bool {
    if env::var_os("TMUX").is_some() {
        debug!(" -> $TMUX is set: running inside tmux");
        return true;
    }
    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            if env_val.to_ascii_lowercase().contains("tmux") {
                debug!(" -> ${env_var}={env_val:?} suggests tmux");
                return true;
            }
        }
    }
    false
}

/// Custom env var storing how deeply tmux is nested. Starts at 1 when there's no nesting.
pub fn get_tmux_nest_count() -> u32 {
    std::env::var("TMUX_NEST_COUNT")
        .map(|s| str::parse(&s).unwrap_or(1))
        .unwrap_or(1)
}

/// Determine whether we're in SSH.
pub fn is_ssh() -> bool {
    for env_var in ["SSH_CLIENT", "SSH_CONNECTION"] {
        if env::var(env_var).is_ok() {
            debug!(" -> this seems to be under SSH");
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_esc_seq_doubles_per_nest_level() {
        assert_eq!(get_esc_seq(0), "\u{1b}");
        assert_eq!(get_esc_seq(1), "\u{1b}\u{1b}");
        assert_eq!(get_esc_seq(2), "\u{1b}".repeat(4));
        assert_eq!(get_esc_seq(3), "\u{1b}".repeat(8));
    }

    #[test]
    fn get_tmux_header_and_tail_nest_two_levels() {
        assert_eq!(get_tmux_header(2), "\u{1b}Ptmux;\u{1b}\u{1b}Ptmux;");
        assert_eq!(get_tmux_tail(2), "\u{1b}\u{1b}\\\u{1b}\\");
    }

    #[test]
    fn get_tmux_header_zero_nest_is_empty() {
        assert_eq!(get_tmux_header(0), "");
        assert_eq!(get_tmux_tail(0), "");
    }
}
