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
use termimad::{CompoundStyle, MadSkin};

pub trait SkinEntry {
    fn print_fg(&self);
    fn print_bg(&self);
    fn print_string(&self, string: &str);
    fn write(&self, f: &mut fmt::Formatter<'_>, string: &str) -> fmt::Result;
}

impl SkinEntry for ObjectStyle {
    fn print_fg(&self) {
        if let Some(c) = self.fg_color {
            print!("{}", Colored::Fg(c));
        }
    }
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

Skin! {
    tree: gray(5), None;
    file: gray(18), None;
    directory: Some(Blue), None; {Bold}
    exe: Some(Cyan), None;
    link: Some(Magenta), None;
    pruning: gray(17), None; {Italic}
    permissions: gray(15), None;
    selected_line: None, gray(3);
    size_bar: gray(15), rgb(117, 80, 123);
    size_no_bar: gray(15), gray(2);
    char_match: rgb(78, 154, 8), None;
    file_error: Some(Red), None;
    flag_label: gray(15), gray(1);
    flag_value: Some(Blue), gray(1);
    input: Some(White), None;
    spinner: gray(10), gray(2);
    status_error: Some(Red), gray(2);
    status_normal: Some(White), gray(2);
    scrollbar_track: rgb(80, 50, 0), None;
    scrollbar_thumb: rgb(255, 187, 0), None;
    help_paragraph: gray(20), None;
    help_bold: rgb(255, 187, 0), None; {Bold}
    help_italic: Some(Magenta), rgb(30, 30, 40); {Italic}
    help_code: gray(21), gray(3);
    help_headers: rgb(255, 187, 0), None;
    help_table_border: rgb(84, 72, 29), None;
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
        ms.table_border = CompoundStyle::from(self.help_table_border.clone());
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
