use {
    crokey::crossterm::style::Color,
    termimad::{gray, MadSkin},
};

/// build a termimad skin for cli output (mostly
/// for the install process)
pub fn make_cli_mad_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.set_headers_fg(Color::AnsiValue(178));
    skin.inline_code.set_bg(gray(2));
    skin.inline_code.set_fg(gray(18));
    skin.code_block.set_bg(gray(2));
    skin.code_block.set_fg(gray(18));
    skin.italic.set_fg(Color::Magenta);
    skin
}
