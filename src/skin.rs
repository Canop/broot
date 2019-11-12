/// Defines the Skin structure with its default value.
///
/// A skin is a collection of termimad compound_style
use std::{
    collections::HashMap,
    fmt,
};

use crossterm::{
    style::{
        Attribute::*,
        Color::AnsiValue,
        Color::{self, *},
    },
};
use termimad::{Alignment, CompoundStyle, LineStyle, MadSkin};

macro_rules! Skin {
    (
        $($name:ident: $fg:expr, $bg:expr; $({$a:expr})*)*
    ) => {
        pub struct Skin {
            $(pub $name: CompoundStyle,)*
        }
        impl Skin {
            /// build a skin without any terminal control character (for file output)
            pub fn no_term() -> Skin {
                Skin {
                    $($name: CompoundStyle::default(),)*
                }
            }
            /// build a skin with some entry overloaded by configuration
            pub fn create(mut skin_conf: HashMap<String, CompoundStyle>) -> Skin {
                Skin {
                    $($name: skin_conf.remove(stringify!($name)).unwrap_or(CompoundStyle::new(
                        $fg,
                        $bg,
                        [$($a),*].to_vec(),
                    )),)*
                }
            }
        }
        impl Clone for Skin {
            fn clone(&self) -> Self {
                Skin {
                    $($name: self.$name.clone(),)*
                }
            }
        }
    }
}

pub fn gray(level: u8) -> Option<Color> {
    Some(AnsiValue(0xE8 + level))
}

pub fn rgb(r: u8, g: u8, b: u8) -> Option<Color> {
    Some(Rgb { r, g, b })
}

pub fn ansi(v: u8) -> Option<Color> {
    Some(AnsiValue(v))
}

// Gold alternative: use 178 for boldish and italic/code is 229
// Orange alternative: boldish is 208 and italic/code is 222
Skin! {
    tree: gray(5), None;
    file: gray(18), None;
    directory: Some(Blue), None; {Bold}
    exe: Some(Cyan), None;
    link: Some(Magenta), None;
    pruning: gray(17), None; {Italic}
    permissions: gray(15), None;
    dates: ansi(109), None;
    selected_line: None, gray(4);
    size_bar: Some(White), gray(2);
    char_match: Some(Green), None;
    file_error: Some(Red), None;
    flag_label: gray(15), gray(1);
    flag_value: ansi(178), gray(1); {Bold}
    input: Some(White), None;
    status_error: gray(22), ansi(124);
    status_job: ansi(220), gray(5);
    status_normal: gray(20), gray(3);
    status_italic: ansi(178), None;
    status_bold: ansi(178), None; {Bold}
    status_code: ansi(229), gray(5);
    status_ellipsis: gray(19), gray(1);
    scrollbar_track: gray(7), None;
    scrollbar_thumb: gray(22), None;
    help_paragraph: gray(20), None;
    help_bold: ansi(178), None; {Bold}
    help_italic: ansi(229), None; {Italic}
    help_code: gray(21), gray(3);
    help_headers: ansi(178), None;
    help_table_border: ansi(239), None;
}


impl Skin {
    /// build a MadSkin, which will be used for markdown formatting
    /// (for the help screen) by applying the `help_*` entries
    /// of the skin.
    pub fn to_mad_skin(&self) -> MadSkin {
        let mut ms = MadSkin::default();
        ms.paragraph.compound_style = CompoundStyle::from(self.help_paragraph.clone());
        ms.inline_code = CompoundStyle::from(self.help_code.clone());
        ms.code_block.compound_style = ms.inline_code.clone();
        ms.bold = CompoundStyle::from(self.help_bold.clone());
        ms.italic = CompoundStyle::from(self.help_italic.clone());
        ms.table = LineStyle {
            compound_style: CompoundStyle::from(self.help_table_border.clone()),
            align: Alignment::Center,
        };
        if let Some(c) = self.help_headers.get_fg() {
            ms.set_headers_fg(c);
        }
        ms.scrollbar
            .track
            .set_compound_style(CompoundStyle::from(self.scrollbar_track.clone()));
        ms.scrollbar
            .thumb
            .set_compound_style(CompoundStyle::from(self.scrollbar_thumb.clone()));
        ms
    }
}

impl fmt::Debug for Skin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Skin")
    }
}
