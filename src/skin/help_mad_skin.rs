use {
    super::StyleMap,
    termimad::{Alignment, LineStyle, MadSkin},
};


/// build a MadSkin, which will be used for markdown formatting
/// for the help screen by applying the `help_*` entries
/// of the skin.
pub fn make_help_mad_skin(skin: &StyleMap) -> MadSkin {
    let mut ms = MadSkin::default();
    ms.paragraph.compound_style = skin.help_paragraph.clone();
    ms.inline_code = skin.help_code.clone();
    ms.code_block.compound_style = ms.inline_code.clone();
    ms.bold = skin.help_bold.clone();
    ms.italic = skin.help_italic.clone();
    ms.table = LineStyle::new(
        skin.help_table_border.clone(),
        Alignment::Center,
    );
    if let Some(c) = skin.help_headers.get_fg() {
        ms.set_headers_fg(c);
    }
    if let Some(c) = skin.help_headers.get_bg() {
        ms.set_headers_bg(c);
    }
    ms.bullet
        .set_compound_style(ms.paragraph.compound_style.clone());
    ms.scrollbar
        .track
        .set_compound_style(skin.scrollbar_track.clone());
    ms.scrollbar
        .thumb
        .set_compound_style(skin.scrollbar_thumb.clone());
    ms
}

