#[macro_use]
extern crate cli_log;

pub mod app;
pub mod browser;
pub mod cli;
pub mod command;
pub mod conf;
pub mod content_search;
pub mod content_type;
pub mod display;
pub mod errors;
pub mod file_sum;
pub mod flag;
pub mod git;
pub mod help;
pub mod hex;
pub mod icon;
pub mod image;
pub mod keys;
pub mod kitty;
pub mod launchable;
pub mod path;
pub mod pattern;
pub mod permissions;
pub mod preview;
pub mod print;
pub mod shell_install;
pub mod skin;
pub mod stage;
pub mod syntactic;
pub mod task_sync;
pub mod terminal;
pub mod tree;
pub mod tree_build;
pub mod tty;
pub mod verb;
pub mod watcher;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub mod filesystems;

#[cfg(unix)]
pub mod net;

#[cfg(any(
    target_os = "windows",
    all(
        unix,
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "android")
    )
))]
pub mod trash;
