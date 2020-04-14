//! This module is where is defined whether broot
//! writes on stdout, on stderr or elsewhere. It also provides helper
//! structs for io.
use std::{
    io::BufWriter,
};

/// the type used by all GUI writing functions
pub type W = BufWriter<std::io::Stderr>;

/// return the writer used by the application
pub fn writer() -> W {
    BufWriter::new(std::io::stderr())
}

