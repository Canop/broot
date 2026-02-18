use {
};

pub enum CommandNumberOnMultipleSelection {
    OnlyOneCommand,
    OneCommandPerSelection,
}

#[derive(Debug)]
pub enum VerbArgFlag {
    PathFromParent,
    PathFromDirectory,
    Path,
    Theme,
    JsonArray,
    FullLines,
    CommaSeparated,
    SpaceSeparated,
    AbsolutePath,
}

