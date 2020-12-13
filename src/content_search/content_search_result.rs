

/// result of a full text search
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentSearchResult {

    /// the needle has been found at the given pos
    Found {
        pos: usize,
    },

    /// the needle hasn't been found
    NotFound, // no match

    /// the file wasn't searched because it's binary or too big
    NotSuitable,
}

impl ContentSearchResult {
    pub fn is_found(self) -> bool {
        matches!(self, Self::Found {..})
    }
}
