use {
    smallvec::SmallVec,
};

/// a vector of indexes of the matching characters (not bytes)
pub type Pos = SmallVec<[usize; 8]>;


