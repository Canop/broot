// #![ allow( dead_code, unused_imports ) ]

#[macro_use]
extern crate crossbeam;
#[macro_use]
extern crate minimad;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_regex;
// #[macro_use]
// extern crate phf;

#[macro_use]
mod time;

#[macro_use]
pub mod display;

pub mod app;
pub mod browser;
pub mod clap;
pub mod cli;
pub mod clipboard;
pub mod command;
pub mod conf;
pub mod content_search;
pub mod errors;
pub mod file_sum;
pub mod flag;
pub mod git;
pub mod hex;
pub mod help;
pub mod keys;
pub mod image;
pub mod launchable;

#[cfg(feature="client-server")]
pub mod net;

pub mod path;
pub mod path_anchor;
pub mod pattern;
pub mod permissions;
pub mod preview;
pub mod print;
pub mod shell_install;
pub mod skin;
pub mod syntactic;
pub mod task_sync;
pub mod tree;
pub mod tree_build;
pub mod verb;
