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

/// Determine whether we're in tmux.
#[allow(unreachable_code)]
pub fn is_tmux() -> bool {
    for env_var in ["TERM", "TERMINAL"] {
        if let Ok(env_val) = env::var(env_var) {
            if env_val.to_ascii_lowercase().contains("tmux") {
                debug!(" -> this terminal seems to be Tmux");
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
