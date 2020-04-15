
use {
    directories::UserDirs,
    regex::{self, Captures, Regex},
    std::{
        collections::HashMap,
        path::Path,
    },
};

/// build a usable path from a user input
fn path_from(
    source: PathSource,
    input: &str,
    replacement_map: &HashMap<String, String>,
) -> String {
    let tilde = regex!(r"^~(/|$)");
    if input.starts_with('/') {
        // if the input starts with a `/`, we use it as is, we don't
        // use the replacement_map
        input.to_string()
    } else if tilde.is_match(input) {
        // if the input starts with `~` as first token, we replace
        // this `~` with the user home directory and  we don't use the
        // replacement map
        tilde.replace(input, |c: &Captures| {
            if let Some(user_dirs) = UserDirs::new() {
                format!(
                    "{}{}",
                    user_dirs.home_dir().to_string_lossy(),
                    &c[1],
                )
            } else {
                warn!("no user dirs found, no expansion of ~");
                c[0].to_string()
            }
        }).to_string()
    } else {
        // we put the input behind the source (the selected directory
        // or its parent) and we normalize so that the user can type
        // paths with `../`
        normalize_path(format!(
            "{}/{}",
            replacement_map.get(source.replacement_map_key()).unwrap(),
            input
        ))
    }
}

/// replace a group in the execution string, using
///  data from the user input and from the selected line
pub fn do_exec_replacement(
    ec: &Captures<'_>,
    replacement_map: &HashMap<String, String>,
) -> String {
    let name = ec.get(1).unwrap().as_str();
    if let Some(cap) = replacement_map.get(name) {
        let cap = cap.as_str();
        debug!("do_exec_replacement cap={:?} with {:?}", &cap, ec.get(2));
        if let Some(fmt) = ec.get(2) {
            match fmt.as_str() {
                "path-from-directory" => path_from(PathSource::Directory, cap, replacement_map),
                "path-from-parent" => path_from(PathSource::Parent, cap, replacement_map),
                _ => format!("invalid format: {:?}", fmt.as_str()),
            }
        } else {
            cap.to_string()
        }
    } else {
        format!("{{{}}}", name)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PathSource {
    Directory,
    Parent,
}
impl PathSource {
    pub fn replacement_map_key(&self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::Parent => "parent",
        }
    }
}

// from a path, build a string usable in a shell command, wrapping
//  it in quotes if necessary (and then escaping internal quotes).
// Don't do unnecessary transformation, so that the produced string
//  is prettier on screen.
pub fn escape_for_shell(path: &Path) -> String {
    let path = path.to_string_lossy();
    if regex!(r"^[\w/.-]*$").is_match(&path) {
        path.to_string()
    } else {
        format!("'{}'", &path.replace('\'', r"'\''"))
    }
}


/// Improve the path to remove and solve .. token.
///
/// This will be removed when this issue is solved: https://github.com/rust-lang/rfcs/issues/2208
///
/// Note that this operation might be a little too optimistic in some cases
/// of aliases but it's probably OK in broot.
pub fn normalize_path(mut path: String) -> String {
    let mut len_before = path.len();
    loop {
        path = regex!(r"/[^/.\\]+/\.\.").replace(&path, "").to_string();
        let len = path.len();
        if len == len_before {
            return path;
        }
        len_before = len;
    }
}
#[cfg(test)]
mod path_normalize_tests {

    use crate::verbs::normalize_path;

    fn check(before: &str, after: &str) {
        assert_eq!(normalize_path(before.to_string()), after.to_string());
    }

    #[test]
    fn test_path_normalization() {
        check("/abc/test/../thing.png", "/abc/thing.png");
        check("/abc/def/../../thing.png", "/thing.png");
        check("/home/dys/test", "/home/dys/test");
        check("/home/dys/..", "/home");
        check("/home/dys/../", "/home/");
        check("/..", "/..");
        check("../test", "../test");
        check("/home/dys/../../../test", "/../test");
        check(
            "/home/dys/dev/broot/../../../canop/test",
            "/home/canop/test",
        );
    }
}
