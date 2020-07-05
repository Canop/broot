use {
    crate::{
        errors::ConfError,
    },
};

// number of columns in enum
const COLS_COUNT: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Col {
    Git,
    Branch,
    Permission,
    Date,
    Size, // includes the size bar in sort mode
    Count,
    Name, // name or subpath, depending on sort
}
impl Col {
    pub fn parse(c: char) -> Result<Self, ConfError> {
        Ok(match c {
            'g' => Self::Git,
            'b' => Self::Branch,
            'd' => Self::Date,
            's' => Self::Size,
            'c' => Self::Count,
            'n' => Self::Name,
            _ => {
                return Err(ConfError::InvalidCols { details: format!("column not recognized : {}", c) });
            }
        })
    }
    pub fn index_in(self, cols: &Cols) -> Option<usize> {
        for (idx, col) in cols.iter().enumerate() {
            if *col==self {
                return Some(idx);
            }
        }
        None
    }
    /// return a Cols which tries to take the s setting into account
    /// but is guaranteed to have every Col exactly once.
    pub fn parse_cols(s: &str) -> Result<Cols, ConfError> {
        let mut cols = DEFAULT_COLS;
        for (idx, c) in s.chars().enumerate() {
            if idx >= COLS_COUNT {
                return Err(ConfError::InvalidCols { details: format!("too long: {:?}", s) });
            }
            // we swap the cols, to ensure both keeps being present
            let col = Col::parse(c)?;
            let dest_idx = col.index_in(&cols).unwrap(); // can't be none by construct
            cols[dest_idx] = cols[idx];
            cols[idx] = col;
        }
        Ok(cols)
    }
}

pub type Cols = [Col;COLS_COUNT];

/// Default column order
pub static DEFAULT_COLS: Cols = [
    Col::Git,
    Col::Size,
    Col::Count,
    Col::Permission,
    Col::Date,
    Col::Branch,
    Col::Name,
];

