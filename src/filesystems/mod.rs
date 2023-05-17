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
    once_cell::sync::Lazy,
    std::sync::Mutex,
};

pub static MOUNTS: Lazy<Mutex<MountList>> = Lazy::new(|| Mutex::new(MountList::new()));

pub fn clear_cache() {
    let mut mount_list = MOUNTS.lock().unwrap();
    mount_list.clear_cache();
}

