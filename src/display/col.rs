use {
    crate::{
        app::AppState,
        errors::ConfError,
        tree::Tree,
    },
    serde::Deserialize,
    std::{
        convert::TryFrom,
        str::FromStr,
    },
};

// number of columns in enum
const COLS_COUNT: usize = 10;

/// One of the "columns" of the tree view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Col {
    /// selection mark, typically a triangle on the selected line
    Mark,

    /// Git file status
    Git,

    /// the branch showing filliation
    Branch,

    /// The filesystem's device id (unix only)
    DeviceId,

    /// file mode and ownership
    Permission,

    /// last modified date
    Date,

    /// file size, including size bar in sort_by_size mode
    Size,

    /// number of files in the directory
    Count,

    /// marks whether the path is staged (not used for now, may be removed)
    Staged,

    /// name of the file, or subpath if relevant due to filtering mode
    Name,
}

pub type Cols = [Col; COLS_COUNT];

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ColsConf {
    /// the old representation, with one character per column
    Compact(String),
    /// the newer representation, with column names in clear
    Array(Vec<String>),
}

/// Default column order
pub static DEFAULT_COLS: Cols = [
    Col::Mark,
    Col::Git,
    Col::DeviceId,
    Col::Size,
    Col::Date,
    Col::Permission,
    Col::Count,
    Col::Branch,
    Col::Staged,
    Col::Name,
];

impl FromStr for Col {
    type Err = ConfError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_ref() {
            "m" | "mark" => Ok(Self::Mark),
            "g" | "git" => Ok(Self::Git),
            "dev" | "device" | "device-id" => Ok(Self::DeviceId),
            "b" | "branch" => Ok(Self::Branch),
            "p" | "permission" => Ok(Self::Permission),
            "d" | "date" => Ok(Self::Date),
            "s" | "size" => Ok(Self::Size),
            "c" | "count" => Ok(Self::Count),
            "staged" => Ok(Self::Staged),
            "n" | "name" => Ok(Self::Name),
            _ => Err(ConfError::InvalidCols {
                details: format!("column not recognized : {s}"),
            }),
        }
    }
}

impl Col {
    /// return the index of the column among the complete Cols ordered list
    pub fn index_in(self, cols: &Cols) -> Option<usize> {
        for (idx, col) in cols.iter().enumerate() {
            if *col == self {
                return Some(idx);
            }
        }
        None
    }
    /// tell whether this column should have an empty character left
    pub fn needs_left_margin(self) -> bool {
        match self {
            Col::Mark => false,
            Col::Git => false,
            Col::DeviceId => true,
            Col::Size => true,
            Col::Date => true,
            Col::Permission => true,
            Col::Count => false,
            Col::Branch => false,
            Col::Staged => false,
            Col::Name => false,
        }
    }
    pub fn is_visible(
        self,
        tree: &Tree,
        _app_state: Option<&AppState>,
    ) -> bool {
        let tree_options = &tree.options;
        match self {
            Col::Mark => tree_options.show_selection_mark,
            Col::Git => tree.git_status.is_some(),
            Col::DeviceId => tree_options.show_device_id,
            Col::Size => tree_options.show_sizes,
            Col::Date => tree_options.show_dates,
            Col::Permission => tree_options.show_permissions,
            Col::Count => tree_options.show_counts,
            Col::Branch => true,
            //Col::Staged => app_state.map_or(false, |a| !a.stage.is_empty()),
            Col::Staged => false,
            Col::Name => true,
        }

    }
}

impl TryFrom<&ColsConf> for Cols {
    type Error = ConfError;
    fn try_from(cc: &ColsConf) -> Result<Self, Self::Error> {
        match cc {
            ColsConf::Compact(s) => parse_cols_single_str(s),
            ColsConf::Array(arr) => parse_cols(arr),
        }
    }
}

/// return a Cols which tries to take the s setting into account
/// but is guaranteed to have every Col exactly once.
#[allow(clippy::ptr_arg)] // &[String] won't compile on all platforms
pub fn parse_cols(arr: &Vec<String>) -> Result<Cols, ConfError> {
    let mut cols = DEFAULT_COLS;
    for (idx, s) in arr.iter().enumerate() {
        if idx >= COLS_COUNT {
            return Err(ConfError::InvalidCols {
                details: format!("too long: {arr:?}"),
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
    parse_cols(&s.chars().map(String::from).collect())
}

