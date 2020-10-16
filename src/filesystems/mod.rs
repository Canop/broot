//! The whole module is only available on linux now

mod filesystems_state;
mod mount_list;

pub use {
    filesystems_state::FilesystemState,
    mount_list::MountList,
};

use {
    std::sync::Mutex,
    crossterm::{
        style::Color,
    },
};

lazy_static! {
    static ref MOUNTS: Mutex<MountList> = Mutex::new(MountList::new());
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
    debug_assert!(share>=0.0 && share <=1.0);
    let idx = (share * SHARE_COLORS.len() as f64) as usize;
    if idx >= SHARE_COLORS.len() {
        SHARE_COLORS[SHARE_COLORS.len()-1]
    } else {
        SHARE_COLORS[idx]
    }
}
