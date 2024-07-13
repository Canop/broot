mod dir_view;
mod preview;
mod preview_transformer;
mod preview_state;
mod zero_len_file_view;

pub use {
    dir_view::DirView,
    preview::Preview,
    preview_transformer::*,
    preview_state::PreviewState,
    zero_len_file_view::ZeroLenFileView,
};

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewMode {

    /// image
    Image,

    /// show the content as text, with syntax coloring if
    /// it makes sense. Fails if the file isn't in UTF8
    Text,

    /// show the content of the file as hex
    Hex,
}
