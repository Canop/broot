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

#[macro_use]
mod time;

#[macro_use]
pub mod display;

pub mod app;
pub mod browser;
pub mod clap;
pub mod cli;
pub mod command;
pub mod conf;
pub mod errors;
pub mod file_sizes;
pub mod git;
pub mod help;
pub mod keys;
pub mod launchable;
pub mod path;
pub mod pattern;
pub mod permissions;
pub mod print;
pub mod selection_type;
pub mod shell_install;
pub mod skin;
pub mod task_sync;
pub mod tree;
pub mod tree_build;
pub mod verb;
