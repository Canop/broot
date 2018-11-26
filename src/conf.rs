
use std::io;
use std::fs;
use toml::{self, Value};
use std::result::Result;

use custom_error::custom_error;

custom_error! {pub FileParseError
    Io{source: io::Error}         = "unable to read from the file",
    Toml{source: toml::de::Error} = "unable to parse TOML",
}

// what's needed to handle a verb
#[derive(Debug)]
pub struct VerbConf {
    pub name: String,
    pub cmd: String,
    pub exec_string: String,
}


// The parsed sanitized conf
#[derive(Debug)]
pub struct Conf {
    verbs: Vec<VerbConf>,
}

impl Conf {
    pub fn from_file(filename: &str) -> Result<Conf, FileParseError> {
        let data = fs::read_to_string(filename)?;
        let root: Value = data.parse::<Value>()?;
        println!("root: {:?}", &root);
        let verbs: Vec<VerbConf> = vec!();
        println!("verbs: {:?}", &verbs);
        Ok(Conf{
            verbs
        })
    }
}
