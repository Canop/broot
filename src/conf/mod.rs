use {
    directories::ProjectDirs,
    std::{
        path::PathBuf,
    },
};


mod conf;
mod default_conf;

pub use {
    conf::Conf,
};

/// return the instance of ProjectDirs holding broot's specific paths
pub fn app_dirs() -> ProjectDirs {
    ProjectDirs::from("org", "dystroy", "broot")
        .expect("Unable to find configuration directories")
}

/// return the path to the config directory, based on XDG
pub fn dir() -> PathBuf {
    app_dirs().config_dir().to_path_buf()
}
