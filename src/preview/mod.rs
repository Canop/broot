mod dir_view;
mod preview;
mod preview_state;
mod preview_transformer;
mod unified_diff_view;
mod zero_len_file_view;

pub use {
    dir_view::DirView,
    preview::Preview,
    preview_state::PreviewState,
    preview_transformer::*,
    unified_diff_view::UnifiedDiffView,
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

    /// Show the content with ANSI escape codes
    Tty,

    /// show the changes of the file since the last commit
    Diff,
}
