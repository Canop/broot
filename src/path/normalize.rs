use std::path::{Component, Path, PathBuf};

/// Improve the path to try remove and solve .. token.
///
/// This assumes that `a/b/../c` is `a/c` which might be different from
/// what the OS would have chosen when b is a link. This is OK
/// for broot verb arguments but can't be generally used elsewhere
/// (a more general solution would probably query the FS and just
/// resolve b in case of links).
///
/// This function ensures a given path ending with '/' still
/// ends with '/' after normalization.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let ends_with_slash = path.as_ref()
        .to_str()
        .map_or(false, |s| s.ends_with('/'));
    let mut normalized = PathBuf::new();
    for component in path.as_ref().components() {
        match &component {
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component);
                }
            }
            _ => {
                normalized.push(component);
            }
        }
    }
    if ends_with_slash {
        normalized.push("");
    }
    normalized
}

#[cfg(test)]
mod path_normalize_tests {
    use super::normalize_path;

    fn check(before: &str, after: &str) {
        println!("-----------------\nnormalizing {before:?}");
        // As seen by Stargateur, the test here doesn't work on Windows
        //
        // There are two problems, at least:
        //
        // * strings used for test use the '/' separator. This is a test problem
        // * we do a "end with '/'" test in the tested function. This might
        //   lead to suboptimal interaction on windows
        assert_eq!(normalize_path(before).to_string_lossy(), after);
    }

    #[test]
    fn test_path_normalization() {
        check("/abc/test/../thing.png", "/abc/thing.png");
        check("/abc/def/../../thing.png", "/thing.png");
        check("/home/dys/test", "/home/dys/test");
        check("/home/dys", "/home/dys");
        check("/home/dys/", "/home/dys/");
        check("/home/dys/..", "/home");
        check("/home/dys/../", "/home/");
        check("/..", "/..");
        check("../test", "../test");
        check("/home/dys/../../../test", "/../test");
        check("π/2", "π/2");
        check(
            "/home/dys/dev/broot/../../../canop/test",
            "/home/canop/test",
        );
    }
}
