#![warn(clippy::all)]

use custom_error::custom_error;
use std::io;

use crate::conf;

custom_error! {pub ProgramError
    Io {source: io::Error}           = "IO Error",
    Conf {source: conf::ConfError}   = "Bad configuration",
    ArgParse {bad: String, valid: String} = "{:?} can't be parsed (valid values: {:?}",
}
