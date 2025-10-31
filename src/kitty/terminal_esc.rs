pub fn get_esc_seq(tmux_nest_count: u32) -> String {
    "\u{1b}".repeat(2usize.pow(tmux_nest_count))
}

pub fn get_tmux_header(tmux_nest_count: u32) -> String {
    let mut header: String = "".into();
    for i in 0..tmux_nest_count {
        header.push_str(&"\u{1b}".repeat(2usize.pow(i)));
        header.push_str("Ptmux;");
    }
    header
}

pub fn get_tmux_tail(tmux_nest_count: u32) -> String {
    let mut tail: String = "".into();
    for i in (0..tmux_nest_count).rev() {
        tail.push_str(&"\u{1b}".repeat(2usize.pow(i)));
        tail.push('\\');
    }
    tail
}
