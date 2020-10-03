
use {
    super::*,
    crate::{
        content_search::*,
    },
    regex,
    std::{
        io::{self, BufReader, BufRead},
        fmt,
        fs::File,
        path::Path,
    },
};

/// A regex for searching in file content
#[derive(Debug, Clone)]
pub struct ContentRegexPattern {
    rex: regex::Regex,
    flags: String,
}

impl fmt::Display for ContentRegexPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cr/{}/{}", self.rex, self.flags)
    }
}

impl ContentRegexPattern {

    pub fn from(pat: &str, flags: &str) -> Result<Self, PatternError> {
        Ok(Self {
            rex: super::build_regex(pat, flags)?,
            flags: flags.to_string(),
        })
    }

    pub fn to_regex_parts(&self) -> (String, String) {
        (self.rex.to_string(), self.flags.clone())
    }

    // TODO optimize with regex::bytes ?
    fn has_match(&self, path: &Path) -> io::Result<bool> {
        for line in BufReader::new(File::open(path)?).lines() {
            if self.rex.is_match(line?.as_str()) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        if !candidate.regular_file || is_path_binary(&candidate.path) {
            return None;
        }
        match self.has_match(&candidate.path) {
            Ok(true) => Some(1),
            Ok(false) => None,
            Err(e) => {
                debug!("error while scanning {:?} : {:?}", &candidate.path, e);
                None
            }
        }
    }

    pub fn try_get_content_match(
        &self,
        path: &Path,
        desired_len: usize,
    ) -> io::Result<Option<ContentMatch>> {
        for line in BufReader::new(File::open(path)?).lines() {
            let line = line?;
            if let Some(regex_match) = self.rex.find(line.as_str()) {
                return Ok(Some(ContentMatch::build(
                    line.as_bytes(),
                    regex_match.start(),
                    regex_match.as_str(),
                    desired_len,
                )));
            }
        }
        Ok(None)
    }

    pub fn get_content_match(
        &self,
        path: &Path,
        desired_len: usize,
    ) -> Option<ContentMatch> {
        self.try_get_content_match(path, desired_len).ok().flatten()
    }
}

