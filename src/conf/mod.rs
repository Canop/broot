use {
    crate::path::untilde,
    directories,
    once_cell::sync::Lazy,
    std::path::{Path, PathBuf},
};

mod conf;
mod default;
mod default_flags;
mod format;
pub mod file_size;
mod import;
mod verb_conf;

pub use {
    conf::Conf,
    default::write_default_conf_in,
    default_flags::*,
    format::*,
    import::*,
    verb_conf::VerbConf,
};


/// return the instance of ProjectDirs holding broot's specific paths
pub fn app_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("org", "dystroy", "broot")
        .expect("Unable to find configuration directories")
}

fn env_conf_dir() -> Option<PathBuf> {
    std::env::var("BROOT_CONFIG_DIR")
        .ok()
        .as_deref()
        .map(untilde)
}

#[cfg(not(target_os = "macos"))]
fn find_conf_dir() -> PathBuf {
    env_conf_dir()
        .unwrap_or_else(|| app_dirs().config_dir().to_path_buf())
}

#[cfg(target_os = "macos")]
fn find_conf_dir() -> PathBuf {
    if let Some(env_dir) = env_conf_dir() {
        return env_dir;
    } else if let Some(user_dirs) = directories::UserDirs::new() {
        // We first search in ~/.config/broot which should be the preferred solution
        let preferred = user_dirs.home_dir().join(".config/broot");
        if preferred.exists() {
            return preferred;
        }
        // The directories crate has a non usual choice of config directory,
        // especially for a CLI application. We use it only when
        // the preferred directory doesn't exist and this one exists.
        // See https://github.com/Canop/broot/issues/103
        let second_choice = app_dirs().config_dir().to_path_buf();
        if second_choice.exists() {
            // An older version of broot was used to write the
            // config, we don't want to lose it.
            return second_choice;
        }
        // Either the config has been scraped or it's a new installation
        return preferred;
    } else {
        // there's no home. There are probably other problems too but here we
        // are just looking for a place for our config, not for a shelter for all
        // so the default will do
        app_dirs().config_dir().to_path_buf()
    }
}

static CONF_DIR: Lazy<PathBuf> = Lazy::new(find_conf_dir);

/// return the path to the config directory
pub fn dir() -> &'static Path {
    &CONF_DIR
}
