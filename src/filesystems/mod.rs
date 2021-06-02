//! The whole module is only available on unix now

mod filesystems_state;
mod mount_list;
mod mount_space_display;

pub use {
    filesystems_state::FilesystemState,
    mount_list::MountList,
    mount_space_display::MountSpaceDisplay,
};

use {
    crossterm::style::Color,
    once_cell::sync::Lazy,
    std::sync::Mutex,
};

pub static MOUNTS: Lazy<Mutex<MountList>> = Lazy::new(|| Mutex::new(MountList::new()));

pub fn clear_cache() {
    let mut mount_list = MOUNTS.lock().unwrap();
    mount_list.clear_cache();
}

static SHARE_COLORS: &[Color] = &[
    Color::AnsiValue(28),
    Color::AnsiValue(29),
    Color::AnsiValue(29),
    Color::AnsiValue(29),
    Color::AnsiValue(29),
    Color::AnsiValue(100),
    Color::AnsiValue(136),
    Color::AnsiValue(172),
    Color::AnsiValue(166),
    Color::AnsiValue(196),
];

pub fn share_color(share: f64) -> Color {
    debug_assert!((0.0..=1.0).contains(&share));
    let idx = (share * SHARE_COLORS.len() as f64) as usize;
    if idx >= SHARE_COLORS.len() {
        SHARE_COLORS[SHARE_COLORS.len() - 1]
    } else {
        SHARE_COLORS[idx]
    }
}
