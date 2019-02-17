use std::io::{self, Write};

use crate::screens::Screen;

use std::collections::HashMap;
use termion::color::{self, *};

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
    status_normal: White, AnsiValue::grayscale(2)
    status_error: Red, AnsiValue::grayscale(2)
    tree: AnsiValue::grayscale(5), Reset
    selected_line: Reset, AnsiValue::grayscale(2)
    permissions: AnsiValue::grayscale(15), Reset
    size_text: AnsiValue::grayscale(15), Reset
    size_bar_full: Reset, Magenta
    size_bar_void: Reset, AnsiValue::grayscale(2)
    file: White, Reset
    directory: LightBlue, Reset
    char_match: Green, Reset
    link: LightMagenta, Reset
    file_error: Red, Reset
    unlisted: AnsiValue::grayscale(13), Reset
    input: White, Reset
    flag_label: AnsiValue::grayscale(14), AnsiValue::grayscale(1)
    flag_value: AnsiValue::grayscale(16), AnsiValue::grayscale(1)
    code: Reset, AnsiValue::grayscale(2)
    table_border: AnsiValue::grayscale(8), Reset
    spinner: AnsiValue::grayscale(10), AnsiValue::grayscale(2)
}


