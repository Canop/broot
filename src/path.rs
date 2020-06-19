use {
    crate::{
        path_anchor::PathAnchor,
    },
    directories::UserDirs,
    regex::{self, Captures},
    std::{
        collections::HashMap,
        path::{Component, Path, PathBuf},
    },
};

/// build a usable path from a user input which may be absolute
/// (if it starts with / or ~) or relative to the supplied base_dir.
/// (we might want to try detect windows drives in the future, too)
///
pub fn path_from<P: AsRef<Path>>(base_dir: P, anchor: PathAnchor, input: &str) -> PathBuf {
    let tilde = regex!(r"^~(/|$)");
    if input.starts_with('/') {
        // if the input starts with a `/`, we use it as is
        input.into()
    } else if tilde.is_match(input) {
        // if the input starts with `~` as first token, we replace
        // this `~` with the user home directory
        PathBuf::from(
            &*tilde
                .replace(input, |c: &Captures| {
                    if let Some(user_dirs) = UserDirs::new() {
                        format!("{}{}", user_dirs.home_dir().to_string_lossy(), &c[1],)
                    } else {
                        warn!("no user dirs found, no expansion of ~");
                        c[0].to_string()
                    }
                })
        )
    } else {
        // we put the input behind the source (the selected directory
        // or its parent) and we normalize so that the user can type
        // paths with `../`
        let base_dir = match anchor {
            PathAnchor::Parent => base_dir.as_ref().parent().unwrap_or(base_dir.as_ref()).to_path_buf(),
            _ => closest_dir(base_dir.as_ref()),
        };
        normalize_path(base_dir.join(input))
    }
}

pub fn path_str_from<P: AsRef<Path>>(base_dir: P, input: &str) -> String {
    path_from(base_dir, PathAnchor::Unspecified, input).to_string_lossy().to_string()
}

/// return the closest enclosing directory
pub fn closest_dir(mut path: &Path) -> PathBuf {
    loop {
        if path.exists() && path.is_dir() {
            return path.to_path_buf();
        }
        match path.parent() {
            Some(parent) => path = parent,
            None => {
                debug!("no existing parent"); // unexpected
                return path.to_path_buf();
            }
        }
    }
}

/// replace a group in the execution string, using
///  data from the user input and from the selected line
pub fn do_exec_replacement(ec: &Captures<'_>, replacement_map: &HashMap<String, String>) -> String {
    let name = ec.get(1).unwrap().as_str();
    if let Some(repl) = replacement_map.get(name) {
        if let Some(fmt) = ec.get(2) {
            match fmt.as_str() {
                "path-from-directory" => path_str_from(replacement_map.get("directory").unwrap(), repl),
                "path-from-parent" => path_str_from(replacement_map.get("parent").unwrap(), repl),
                _ => format!("invalid format: {:?}", fmt.as_str()),
            }
        } else {
            repl.to_string()
        }
    } else {
        format!("{{{}}}", name)
    }
}

/// from a path, build a string usable in a shell command, wrapping
///  it in quotes if necessary (and then escaping internal quotes).
/// Don't do unnecessary transformation, so that the produced string
///  is prettier on screen.
pub fn escape_for_shell(path: &Path) -> String {
    let path = path.to_string_lossy();
    if regex!(r"^[\w/.-]*$").is_match(&path) {
        path.to_string()
    } else {
        format!("'{}'", &path.replace('\'', r"'\''"))
    }
}

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
    let ends_with_slash = path.as_ref().to_str().map_or(false, |s| s.ends_with('/'));
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
        println!("-----------------\nnormalizing {:?}", before);
        assert_eq!(normalize_path(before.to_string()).to_string_lossy(), after);
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
