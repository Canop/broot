use {
    super::*,
    directories::UserDirs,
    ahash::AHashMap,
    lazy_regex::*,
    std::path::{Path, PathBuf},
};

pub static TILDE_REGEX: Lazy<Regex> = lazy_regex!(r"^~(/|$)");

/// If the input has a tilde as first complete element, replace it
/// with the user's home directory. Return the input as a path without
/// transformation in other cases
pub fn untilde(input: &str) -> PathBuf {
    PathBuf::from(
        &*TILDE_REGEX
            .replace(input, |c: &Captures| {
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
            })
    )
}

/// Build a usable path from a user input which may be absolute
/// (if it starts with / or ~) or relative to the supplied base_dir.
/// (we might want to try detect windows drives in the future, too)
pub fn path_from<P: AsRef<Path> + std::fmt::Debug>(
    base_dir: P,
    anchor: PathAnchor,
    input: &str,
) -> PathBuf {
    if input.starts_with('/') {
        // if the input starts with a `/`, we use it as is
        input.into()
    } else if TILDE_REGEX.is_match(input) {
        // if the input starts with `~` as first token, we replace
        // this `~` with the user home directory
        untilde(input)
    } else {
        // we put the input behind the source (the selected directory
        // or its parent) and we normalize so that the user can type
        // paths with `../`
        let base_dir = match anchor {
            PathAnchor::Parent => base_dir
                .as_ref()
                .parent()
                .unwrap_or_else(|| base_dir.as_ref())
                .to_path_buf(),
            _ => closest_dir(base_dir.as_ref()),
        };
        normalize_path(base_dir.join(input))
    }
}

pub fn path_str_from<P: AsRef<Path> + std::fmt::Debug>(
    base_dir: P,
    input: &str,
) -> String {
    path_from(base_dir, PathAnchor::Unspecified, input)
        .to_string_lossy()
        .to_string()
}

/// Replace a group in the execution string, using
///  data from the user input and from the selected line
pub fn do_exec_replacement(
    ec: &Captures<'_>,
    replacement_map: &AHashMap<String, String>,
) -> String {
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
        format!("{{{name}}}")
    }
}
