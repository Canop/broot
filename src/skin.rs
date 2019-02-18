/// Defines the Skin structure with its defautl value.
///
/// A skin is a collection of skin entries and the
/// store of all color changing TTY codes used by
/// the application. It can be changed by configuration.
use std::collections::HashMap;
use termion::color::{self, *};

/// a Skin entry is a couple of strings, one for the foreground
/// and one for the background, each one made of TTY escape
/// codes defining color changes.
pub struct SkinEntry {
    pub fg: String,
    pub bg: String,
}

impl SkinEntry {
    pub fn fgbg(&self) -> String {
        format!("{}{}", self.fg, self.bg)
    }
}

macro_rules! Skin {
    (
        $($name:ident: $fg:expr, $bg:expr)*
    ) => {
        pub struct Skin {
            $(pub $name: SkinEntry,)*
            pub reset: SkinEntry,
        }
        impl Skin {
            pub fn create(mut skin_conf: HashMap<String, String>) -> Skin {
                Skin {
                    $($name: SkinEntry {
                        fg: skin_conf.remove(&format!("{}_fg", stringify!($name))).unwrap_or(
                            format!("{}", color::Fg($fg)).to_string()
                        ),
                        bg: skin_conf.remove(&format!("{}_bg", stringify!($name))).unwrap_or(
                            format!("{}", color::Bg($bg)).to_string()
                        ),
                    },)*
                    reset: SkinEntry {
                        fg: format!("{}", color::Fg(color::Reset)).to_string(),
                        bg: format!("{}", color::Bg(color::Reset)).to_string(),
                    },
                }
            }
        }
    }
}

Skin! {
    char_match: Green, Reset
    code: Reset, AnsiValue::grayscale(2)
    directory: LightBlue, Reset
    file: White, Reset
    file_error: Red, Reset
    flag_label: AnsiValue::grayscale(12), AnsiValue::grayscale(1)
    flag_value: AnsiValue::grayscale(16), AnsiValue::grayscale(1)
    input: White, Reset
    link: LightMagenta, Reset
    permissions: AnsiValue::grayscale(15), Reset
    selected_line: Reset, AnsiValue::grayscale(3)
    size_bar_full: Reset, Magenta
    size_bar_void: Reset, AnsiValue::grayscale(2)
    size_text: AnsiValue::grayscale(15), Reset
    spinner: AnsiValue::grayscale(10), AnsiValue::grayscale(2)
    status_error: Red, AnsiValue::grayscale(2)
    status_normal: White, AnsiValue::grayscale(2)
    table_border: AnsiValue::grayscale(8), Reset
    tree: AnsiValue::grayscale(5), Reset
    unlisted: AnsiValue::grayscale(13), Reset
}


