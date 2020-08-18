use {
    crate::{
        errors::ConfError,
    },
};

// number of columns in enum
const COLS_COUNT: usize = 8;

/// One of the "columns" of the tree view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Col {
    /// selection mark, typically a triangle on the selected line
    Mark,

    /// Git file status
    Git,

    /// the branch showing filliation
    Branch,

    /// file mode and ownership
    Permission,

    /// last modified date
    Date,

    /// file size, including size bar in sort_by_size mode
    Size,

    /// number of files in the directory
    Count,

    /// name of the file, or subpath if relevant due to filtering mode
    Name,
}

impl Col {
    pub fn parse(c: char) -> Result<Self, ConfError> {
        Ok(match c {
            'm' => Self::Mark,
            'g' => Self::Git,
            'b' => Self::Branch,
            'd' => Self::Date,
            's' => Self::Size,
            'c' => Self::Count,
            'n' => Self::Name,
            _ => {
                return Err(ConfError::InvalidCols {
                    details: format!("column not recognized : {}", c),
                });
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
                return Err(ConfError::InvalidCols {
                    details: format!("too long: {:?}", s),
                });
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
    Col::Mark,
    Col::Git,
    Col::Size,
    Col::Count,
    Col::Permission,
    Col::Date,
    Col::Branch,
    Col::Name,
];

