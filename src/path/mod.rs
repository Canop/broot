mod anchor;
mod closest;
mod common;
mod from;
mod normalize;
mod special_path;

pub use {
    anchor::*,
    closest::*,
    common::*,
    from::*,
    normalize::*,
    special_path::*,
};

use std::{
    path::Path,
};

/// Check if the path has the given extension (case-insensitive).
///
/// Works with multi-part extensions (e.g. `.tar.gz`) and ignores
/// leading dots in the extension (so `path_has_ext("file.txt", ".txt")` and
/// `path_has_ext("file.txt", "txt")` are equivalent).
/// Dot files (eg. `.gitignore`) are considered to have no extension.
pub fn path_has_ext<P: AsRef<Path>>(
    path: P,
    ext: &str,
) -> bool {
    let Some(name) = path.as_ref().file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    let name = name.as_bytes();
    let ext = ext.trim_start_matches('.').as_bytes();
    name.len() > ext.len() + 1
        && name[name.len() - ext.len() - 1] == b'.'
        && name[name.len() - ext.len()..].eq_ignore_ascii_case(ext)
}

#[test]
fn test_path_has_ext() {
    assert_eq!(path_has_ext("file.txt", "txt"), true);
    assert_eq!(path_has_ext("file.txt", ".txt"), true);
    assert_eq!(path_has_ext("file.", ""), true);
    assert_eq!(path_has_ext("file.tar.gz", "gz"), true);
    assert_eq!(path_has_ext("file.tar.gz", ".gz"), true);
    assert_eq!(path_has_ext("file.tar.gz", "tar.gz"), true);
    assert_eq!(path_has_ext("file.tar.gz", "ar.gz"), false);
    assert_eq!(path_has_ext("file.tar.gz", ".tar.gz"), true);
    assert_eq!(path_has_ext("f.tar.gz", ".tar.gz"), true);
    assert_eq!(path_has_ext("f.tar.gz", "gz"), true);
    assert_eq!(path_has_ext("f.tar.gz", "z"), false);
    assert_eq!(path_has_ext(".tar.gz", ".tar.gz"), false);
    assert_eq!(path_has_ext("tar.gz", "tar.gz"), false);
    assert_eq!(path_has_ext("file.tar.gz", "notar.gz"), false);
    assert_eq!(path_has_ext("file.tar.gz", ".notar.gz"), false);
    assert_eq!(path_has_ext(".gitignore", ""), false);
    assert_eq!(path_has_ext(".gitignore", "gitignore"), false);
}
