use {
    serde::Deserialize,
};

/// one of the types of state that you could
/// find in a panel today
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelStateType {

    /// standard browsing tree
    Tree,

    /// filesystems
    Fs,

    /// help "screen"
    Help,

    /// preview panel, never alone on screen
    Preview,

    /// stage panel, never alone on screen
    Stage,
}
