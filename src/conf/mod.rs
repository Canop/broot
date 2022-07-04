use {
    directories,
    std::path::{Path, PathBuf},
};

mod conf;
mod default;
mod format;
mod import;
mod verb_conf;

pub use {
    conf::Conf,
    default::write_default_conf_in,
    format::*,
    import::*,
    once_cell::sync::Lazy,
    verb_conf::VerbConf,
};


/// return the instance of ProjectDirs holding broot's specific paths
pub fn app_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("org", "dystroy", "broot")
        .expect("Unable to find configuration directories")
}

#[cfg(not(target_os = "macos"))]
fn find_conf_dir() -> PathBuf {
    app_dirs().config_dir().to_path_buf()
}

#[cfg(target_os = "macos")]
fn find_conf_dir() -> PathBuf {
    if let Some(user_dirs) = directories::UserDirs::new() {
        // We first search in ~/.config/broot which should be the prefered solution
        let prefered = user_dirs.home_dir().join(".config/broot");
        if prefered.exists() {
            return prefered;
        }
        // The directories crate has a non usual choice of config directory,
        // especially for a CLI application. We use it only when
        // the prefered directory doesn't exist and this one exists.
        // See https://github.com/Canop/broot/issues/103
        let second_choice = app_dirs().config_dir().to_path_buf();
        if second_choice.exists() {
            // An older version of broot was used to write the
            // config, we don't want to lose it.
            return second_choice;
        }
        // Either the config has been scraped or it's a new installation
        return prefered;
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
    &*CONF_DIR
}
