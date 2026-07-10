use {
    super::StyleMap,
    termimad::{
        Alignment,
        LineStyle,
        MadSkin,
    },
};

/// build a MadSkin, which will be used for markdown formatting
/// for the help screen by applying the `help_*` entries
/// of the skin.
pub fn make_help_mad_skin(skin: &StyleMap) -> MadSkin {
    let mut ms = MadSkin::default();
    ms.paragraph.compound_style = skin.help_paragraph;
    ms.inline_code = skin.help_code;
    ms.code_block.compound_style = ms.inline_code;
    ms.bold = skin.help_bold;
    ms.italic = skin.help_italic;
    ms.table = LineStyle::new(skin.help_table_border, Alignment::Center);
    if let Some(c) = skin.help_headers.get_fg() {
        ms.set_headers_fg(c);
    }
    if let Some(c) = skin.help_headers.get_bg() {
        ms.set_headers_bg(c);
    }
    ms.bullet
        .set_compound_style(ms.paragraph.compound_style);
    ms.scrollbar
        .track
        .set_compound_style(skin.scrollbar_track);
    ms.scrollbar
        .thumb
        .set_compound_style(skin.scrollbar_thumb);
    ms
}
