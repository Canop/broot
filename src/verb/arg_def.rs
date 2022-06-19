use {
    crate::{
        app::SelectionType,
        path::PathAnchor,
    },
};

/// The definition of an argument given to a verb
/// as understood from the invocation pattern
#[derive(Debug, Clone, Copy)]
pub enum ArgDef {
    Path {
        anchor: PathAnchor,
        selection_type: SelectionType,
    },
    Theme,
    Unspecified,
}
