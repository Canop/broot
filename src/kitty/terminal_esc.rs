pub fn get_esc_seq(is_tmux: bool) -> &'static str {
    if is_tmux {
        return "\u{1b}\u{1b}";
    }
    "\u{1b}"
}

#[macro_export]
macro_rules! tmux_write_header {
    ($w:expr) => {
        write!($w, "\u{1b}Ptmux;")
    };
}

#[macro_export]
macro_rules! tmux_write_tail {
    ($w:expr) => {
        write!($w, "\u{1b}\\")
    };
}
