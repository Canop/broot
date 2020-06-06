
use {
    super::Match,
    crate::{
        content_search::{ContentSearchResult, Needle},
    },
    std::{
        fmt::{self, Write},
        path::Path,
    }
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

    /// build a pattern which will later be usable for fuzzy search.
    /// A pattern should be reused
    pub fn from(pat: &str) -> Self {
        Self {
            needle: Needle::new(pat),
        }
    }

    pub fn score_of<P: AsRef<Path>>(&self, path: P) -> Option<i32> {
        match self.needle.search(&path) {
            Ok(ContentSearchResult::Found { .. }) => Some(1),
            Ok(ContentSearchResult::NotFound) => None,
            Ok(ContentSearchResult::NotSuitable) => {
                debug!("{:?} isn't suitable for search", path.as_ref());
                None
            }
            Err(e) => {
                info!("error while searching: {:?}", e);
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

