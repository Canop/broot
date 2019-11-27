//////////////////// UNIX

#[cfg(unix)]
pub mod permissions_unix;

#[cfg(unix)]
pub fn supported() -> bool {
    true
}

#[cfg(unix)]
pub use permissions_unix::*;

//////////////////// WINDOWS

#[cfg(windows)]
pub fn supported() -> bool {
    false
}
