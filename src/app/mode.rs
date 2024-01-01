use {
    serde::Deserialize,
};

/// modes are used when the application is configured to
/// be "modal". If not, the only mode is the `Input` mode.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Input,
    Command,
}
