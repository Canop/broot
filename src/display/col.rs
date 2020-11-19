use {
    crate::{
        errors::ConfError,
    },
    std::str::FromStr,
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

impl FromStr for Col {
    type Err = ConfError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_ref() {
            "m" | "mark" => Ok(Self::Mark),
            "g" | "git"  => Ok(Self::Git),
            "b" | "branch" =>  Ok(Self::Branch),
            "p" | "permission" => Ok(Self::Permission),
            "d" | "date" => Ok(Self::Date),
            "s" | "size" => Ok(Self::Size),
            "c" | "count" => Ok(Self::Count),
            "n" | "name" => Ok(Self::Name),
            _ => Err(ConfError::InvalidCols {
                details: format!("column not recognized : {}", s),
            })
        }
    }
}

impl Col {
    pub fn index_in(self, cols: &Cols) -> Option<usize> {
        for (idx, col) in cols.iter().enumerate() {
            if *col==self {
                return Some(idx);
            }
        }
        None
    }
    // warning: don't change the type of argument of this function or
    // it won't compile on some platforms
    /// return a Cols which tries to take the s setting into account
    /// but is guaranteed to have every Col exactly once.
    pub fn parse_cols(arr: &Vec<String>) -> Result<Cols, ConfError> {
        let mut cols = DEFAULT_COLS;
        for (idx, s) in arr.iter().enumerate() {
            if idx >= COLS_COUNT {
                return Err(ConfError::InvalidCols {
                    details: format!("too long: {:?}", arr),
                });
            }
            // we swap the cols, to ensure both keeps being present
            let col = Col::from_str(s)?;
            let dest_idx = col.index_in(&cols).unwrap(); // can't be none by construct
            cols[dest_idx] = cols[idx];
            cols[idx] = col;
        }
        debug!("cols from conf = {:?}", cols);
        Ok(cols)
    }
    /// return a Cols which tries to take the s setting into account
    /// but is guaranteed to have every Col exactly once.
    pub fn parse_cols_single_str(s: &str) -> Result<Cols, ConfError> {
        Self::parse_cols(&s.chars().map(String::from).collect())
    }
}

pub type Cols = [Col;COLS_COUNT];

/// Default column order
pub static DEFAULT_COLS: Cols = [
    Col::Mark,
    Col::Git,
    Col::Permission,
    Col::Date,
    Col::Size,
    Col::Count,
    Col::Branch,
    Col::Name,
];

