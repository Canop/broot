mod preview;
mod preview_state;

pub use {
    preview::Preview,
    preview_state::PreviewState,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PreviewMode {

    /// image
    Image,

    /// show the content as text, with syntax coloring if
    /// it makes sens. Fails if the file isn't in UTF8
    Text,

    /// show the content of the file as hex
    Hex,
}
