/// Describe whether a command should be executed once per selection or
/// once per command invocation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandCoarity {
    // one command per selection
    PerSelection,
    // One command integrating all selections, levaraging "repeated" or "repeating" patterns to
    // specify how selections are integrated in the command.
    Merged,
}
