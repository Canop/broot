use std::path::Path;
use std::ops::AddAssign;
use walkdir::WalkDir;
use std::fs;


const SIZE_NAMES: &[&str] = &["", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"]; // YB: for when your disk is bigger than 1024 ZB

#[derive(Debug, Copy, Clone)]
pub struct Size(u64);

impl Size {
    pub fn from_file(path: &Path) -> Size {
        Size(
            match fs::metadata(path) {
                Ok(m) => m.len(),
                Err(_) => 0,
            }
        )
    }
    // TODO : make interruptible (writing my own walker will probably
    //  be necessary)
    pub fn from_dir(path: &Path) -> Size {
        Size(
             WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok()
                            .and_then(|f| {
                                    f.metadata().map(|m| m.len()).ok()
                            }))
                .sum()
        )
    }
    /// format a number of bytes as a string
    /// (probably fast enough but not benchmarked)
    pub fn to_string(&self) -> String {
        let mut v = self.0;
        let mut i = 0;
        while v >= 1024 && i < SIZE_NAMES.len()-1 {
            v /= 1024;
            i += 1;
        }
        format!("{}{}", v, &SIZE_NAMES[i])
    }
}

impl From<u64> for Size {
    fn from(s: u64) -> Size {
        Size(s)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, other: Size) {
        *self = Size(self.0 + other.0);
    }
}


//impl Into<u64> for Size {
//    fn into(self) -> u64 {
//        self.0
//    }
//}
