
/// one of the types of state that you could
/// find in a panel today
#[derive(Debug, Clone, Copy)]
pub enum PanelStateType {

    /// The standard browsing tree
    Tree,

    /// the filesystem
    Fs,

    /// The help "screen"
    Help,

    /// The preview panel, never alone on screen
    Preview,

    /// The stage panel, never alone on screen
    Stage,
}
