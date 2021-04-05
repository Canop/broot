
/// one of the three types of state that you could
/// find in a panel today
#[derive(Debug, Clone, Copy)]
pub enum PanelStateType {

    /// The standard browsing tree
    Tree,

    /// The help "screen"
    Help,

    /// The preview panel, never alone on screen
    Preview,
}
