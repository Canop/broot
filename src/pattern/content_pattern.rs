
use {
    super::*,
    crate::{
        content_search::{ContentSearchResult, Needle},
    },
    std::fmt,
};

/// A pattern for searching in file content
#[derive(Debug, Clone)]
pub struct ContentPattern {
    needle: Needle,
}

impl fmt::Display for ContentPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.needle.as_str())
    }
}

impl ContentPattern {

    pub fn from(pat: &str) -> Self {
        Self {
            needle: Needle::new(pat),
        }
    }

    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        if !candidate.file_type.is_file() {
            return None;
        }
        match self.needle.search(&candidate.path) {
            Ok(ContentSearchResult::Found { .. }) => Some(1),
            Ok(ContentSearchResult::NotFound) => None,
            Ok(ContentSearchResult::NotSuitable) => {
                // debug!("{:?} isn't suitable for search", &candidate.path);
                None
            }
            Err(e) => {
                info!("error while scanning {:?} : {:?}", &candidate.path, e);
                None
            }
        }
    }

    /// return the number of results we should find before starting to
    ///  sort them (unless time is runing out).
    pub const fn optimal_result_number(&self, targeted_size: usize) -> usize {
        targeted_size
    }
}

