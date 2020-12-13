//////////////////// UNIX

#[cfg(not(any(target_family = "windows", target_os = "android")))]
pub mod permissions_unix;

#[cfg(not(any(target_family = "windows", target_os = "android")))]
pub use permissions_unix::*;

//////////////////// WINDOWS

#[cfg(windows)]
pub fn supported() -> bool {
    false
}
