use {
    crate::{
        app::SelectionType,
        content_type,
        tree::{TreeLine, TreeLineType},
    },
    serde::Deserialize,
    std::path::Path,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileTypeCondition {
    #[default]
    Any,
    // directory or link to a directory
    Directory,
    File,
    TextFile,
    BinaryFile,
}

impl FileTypeCondition {
    pub fn accepts_path(self, path: &Path) -> bool {
        match self {
            Self::Any => true,
            Self::Directory => path.is_dir(),
            Self::File => path.is_file(),
            Self::TextFile => {
                path.is_file() && matches!(content_type::is_file_text(path), Ok(true))
            }
            Self::BinaryFile => {
                path.is_file() && matches!(content_type::is_file_binary(path), Ok(true))
            }
        }
    }
    pub fn accepts_line(self, line: &TreeLine) -> bool {
        match self {
            Self::Any => true,
            Self::Directory => line.is_dir(),
            Self::File => matches!(line.line_type, TreeLineType::File),
            Self::TextFile => {
                line.is_file() && matches!(content_type::is_file_text(&line.path), Ok(true))
            }
            Self::BinaryFile => {
                line.is_file() && matches!(content_type::is_file_binary(&line.path), Ok(true))
            }
        }
    }
    /// a little clunky, should be used only on well defined cases, like documenting
    /// internals
    pub fn accepts_selection_type(
        self,
        stype: SelectionType,
    ) -> bool {
        match (self, stype) {
            (Self::Any, _) => true,
            (Self::Directory, SelectionType::Directory) => true,
            (Self::File, SelectionType::File) => true,
            _ => false,
        }
    }
}
