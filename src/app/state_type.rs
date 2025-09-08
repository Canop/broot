use serde::{
    Deserialize,
    Serialize,
};

/// one of the types of state that you could
/// find in a panel today
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelStateType {
    /// filesystems
    Fs,

    /// help "screen"
    Help,

    /// preview panel, never alone on screen
    Preview,

    /// stage panel, never alone on screen
    Stage,

    /// content of the trash
    Trash,

    /// standard browsing tree
    Tree,
}
