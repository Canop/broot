
use {
    super::*,
    crate::{
        content_search::*,
    },
    std::{
        fmt,
        path::Path,
    },
};

/// A pattern for searching in file content
#[derive(Debug, Clone)]
pub struct ContentExactPattern {
    needle: Needle,
}

impl fmt::Display for ContentExactPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ContentExactPattern {

    pub fn new(pat: &str, max_file_size: usize) -> Self {
        Self { needle: Needle::new(pat, max_file_size) }
    }

    pub fn as_str(&self) -> &str {
        self.needle.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.needle.is_empty()
    }

    pub fn to_regex_parts(&self) -> (String, String) {
        (regex::escape(self.as_str()), "".to_string())
    }

    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        if !candidate.regular_file {
            return None;
        }
        match self.needle.search(candidate.path) {
            Ok(ContentSearchResult::Found { .. }) => Some(1),
            Ok(ContentSearchResult::NotFound) => None,
            Ok(ContentSearchResult::NotSuitable) => {
                None
            }
            Err(e) => {
                debug!("error while scanning {:?} : {:?}", &candidate.path, e);
                None
            }
        }
    }

    /// get the line of the first match, if any
    pub fn get_match_line_count(
        &self,
        path: &Path,
    ) -> Option<usize> {
        if let Ok(ContentSearchResult::Found { pos }) = self.needle.search(path) {
            line_count_at_pos(path, pos).ok()
        } else {
            None
        }
    }

    pub fn get_content_match(
        &self,
        path: &Path,
        desired_len: usize,
    ) -> Option<ContentMatch> {
        self.needle.get_match(path, desired_len)
    }
}

