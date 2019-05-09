/// Defines the Skin structure with its defautl value.
///
/// A skin is a collection of skin entries and the
/// store of all color changing TTY codes used by
/// the application. It can be changed by configuration.
/// A skin also contains some style change (bold, italic, reset)
/// but they're not configurable right now (they exist mainly
/// so that unstyled output for files can be produced)
use std::collections::HashMap;
use std::fmt;

use crossterm::{Attribute::{self, *}, Color::{self, *}, Colored, Color::AnsiValue, ObjectStyle};

pub trait SkinEntry {
    fn print_fg(&self);
    fn print_bg(&self);
    fn print_string(&self, string: &str);
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
    fn print_string(&self, string: &str) {
        print!("{}", self.apply_to(string));
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
            pub fn create(skin_conf: HashMap<String, String>) -> Skin {
                Skin {
                    $($name: ObjectStyle {
                        fg_color: $fg,
                        bg_color: $bg,
                        attrs: [$($a),*].to_vec()
                    },)*
                }
            }
        }
    }
}

pub fn gray(level: u8) -> Option<Color> {
    Some(AnsiValue(0xE8 + level))
}
pub fn rgb(r: u8, g: u8, b: u8) -> Option<Color> {
    Some(Rgb{r, g, b})
}

Skin! {
    // FIXME some colors to rebuild using Rgb
    char_match: rgb(78, 154, 8), None;
    code: Some(White), gray(2);
    directory: Some(Blue), None; {Bold}
    exe: Some(Cyan), None;
    file: gray(18), None;
    pruning: gray(17), None; {Italic}
    file_error: Some(Red), None;
    flag_label: gray(12), gray(1);
    flag_value: gray(16), gray(1);
    input: Some(White), None;
    link: Some(Magenta), None;
    permissions: gray(15), None;
    selected_line: None, gray(3);
    size_bar_full: Some(White), rgb(117, 80, 123);
    size_bar_void: Some(White), gray(2);
    size_text: gray(15), None;
    spinner: gray(10), gray(2);
    status_error: Some(Red), gray(2);
    status_normal: Some(White), gray(2);
    table_border: gray(8), None;
    tree: gray(5), None;
    unlisted: gray(13), None;
}

pub fn reset() {
    print!("{}", Attribute::Reset);
}
