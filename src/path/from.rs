use {
    super::*,
    directories::UserDirs,
    lazy_regex::*,
    std::path::{
        Path,
        PathBuf,
    },
};

pub static TILDE_REGEX: Lazy<Regex> = lazy_regex!(r"^~(/|$)");

/// If the input has a tilde as first complete element, replace it
/// with the user's home directory. Return the input as a path without
/// transformation in other cases
pub fn untilde(input: &str) -> PathBuf {
    PathBuf::from(&*TILDE_REGEX.replace(input, |c: &Captures| {
        if let Some(user_dirs) = UserDirs::new() {
            format!("{}{}", user_dirs.home_dir().to_string_lossy(), &c[1],)
        } else {
            warn!("no user dirs found, no expansion of ~");
            c[0].to_string()
        }
    }))
}

/// Build a usable path from a user input which may be absolute
/// (if it starts with / or ~) or relative to the supplied `base_dir`.
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
        let base_dir = base_dir.as_ref();
        let base_dir = match anchor {
            PathAnchor::Parent => base_dir.parent().unwrap_or(base_dir),
            PathAnchor::Directory => base_dir,
        };
        normalize_path(closest_dir(base_dir).join(input))
    }
}

pub fn path_str_from<P: AsRef<Path> + std::fmt::Debug>(
    base_dir: P,
    input: &str,
) -> String {
    path_from(base_dir, PathAnchor::Directory, input)
        .to_string_lossy()
        .to_string()
}

