/// Defines the Skin structure with its defautl value.
///
/// A skin is a collection of skin entries which are crossterm
/// objectstyles:
/// - an optional fg color
/// - an optional bg color
/// - a vector of attributes (bold, italic)
use std::collections::HashMap;
use std::fmt;

use crossterm::{
    Attribute::{self, *},
    Color::AnsiValue,
    Color::{self, *},
    Colored, ObjectStyle,
};
use termimad::{Alignment, CompoundStyle, LineStyle, MadSkin};

pub trait SkinEntry {
    fn print_bg(&self);
    fn print_string(&self, string: &str);
    fn write(&self, f: &mut fmt::Formatter<'_>, string: &str) -> fmt::Result;
}

impl SkinEntry for ObjectStyle {
    fn print_bg(&self) {
        if let Some(c) = self.bg_color {
            print!("{}", Colored::Bg(c));
        }
    }
    #[inline(always)]
    fn print_string(&self, string: &str) {
        print!("{}", self.apply_to(string));
    }
    #[inline(always)]
    fn write(&self, f: &mut fmt::Formatter<'_>, string: &str) -> fmt::Result {
        write!(f, "{}", self.apply_to(string))
    }
}

macro_rules! Skin {
    (
        $($name:ident: $fg:expr, $bg:expr; $({$a:expr})*)*
    ) => {
        pub struct Skin {
            $(pub $name: ObjectStyle,)*
        }
        impl Skin {
            // build a skin without any terminal control character (for file output)
            pub fn no_term() -> Skin {
                Skin {
                    $($name: ObjectStyle::new(),)*
                }
            }
            // build a skin with some entry overloaded by configuration
            pub fn create(mut skin_conf: HashMap<String, ObjectStyle>) -> Skin {
                Skin {
                    $($name: skin_conf.remove(stringify!($name)).unwrap_or(ObjectStyle {
                        fg_color: $fg,
                        bg_color: $bg,
                        attrs: [$($a),*].to_vec()
                    }),)*
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

Skin! {
    tree: gray(5), None;
    file: gray(18), None;
    directory: Some(Blue), None; {Bold}
    exe: Some(Cyan), None;
    link: Some(Magenta), None;
    pruning: gray(17), None; {Italic}
    permissions: gray(15), None;
    dates: ansi(109), None;
    selected_line: None, gray(3);
    size_bar: Some(White), Some(DarkBlue);
    size_no_bar: gray(15), gray(2);
    char_match: Some(Green), None;
    file_error: Some(Red), None;
    flag_label: gray(15), gray(1);
    flag_value: Some(Blue), gray(1);
    input: Some(White), None;
    spinner: gray(10), gray(2);
    status_error: Some(Red), gray(2);
    status_normal: Some(White), gray(2);
    scrollbar_track: gray(7), None;
    scrollbar_thumb: ansi(178), None;
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
        ms.code.compound_style = CompoundStyle::from(self.help_code.clone());
        ms.bold = CompoundStyle::from(self.help_bold.clone());
        ms.italic = CompoundStyle::from(self.help_italic.clone());
        ms.table = LineStyle {
            compound_style: CompoundStyle::from(self.help_table_border.clone()),
            align: Alignment::Center,
        };
        if let Some(c) = self.help_headers.fg_color {
            ms.set_headers_fg(c);
        }
        ms.scrollbar.set_track_object_style(&self.scrollbar_track);
        ms.scrollbar.set_thumb_object_style(&self.scrollbar_thumb);
        ms
    }
}

pub fn reset() {
    print!("{}", Attribute::Reset);
}

impl fmt::Debug for Skin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Skin")
    }
}
