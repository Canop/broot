//////////////////// UNIX

#[cfg(unix)]
pub mod permissions_unix;

#[cfg(unix)]
pub fn supported() -> bool {
    true
}

#[cfg(unix)]
pub fn user_name(uid: u32) -> String {
    permissions_unix::user_name(uid)
}

#[cfg(unix)]
pub fn group_name(gid: u32) -> String {
    permissions_unix::group_name(gid)
}

//////////////////// WINDOWS

#[cfg(windows)]
pub fn supported() -> bool {
    false
}

#[cfg(windows)]
pub fn user_name(uid: u32) -> String {
    unreachable!()
}

#[cfg(windows)]
pub fn group_name(gid: u32) -> String {
    unreachable!()
}
