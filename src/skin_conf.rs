use std::result::Result;
use std::ascii::AsciiExt;
use termion::color::{self, *};
use crate::errors::ConfError;

macro_rules! make_parseurs {
    (
        $($name:tt,)*
    ) => {
        pub fn parse_fg(raw: &str) -> String {
            $(
                debug!("comparing {} and {}", raw, stringify!($name));
                if raw.eq_ignore_ascii_case(stringify!($name)) {
                    return format!("{}", Fg($name));
                }
            )*
            return "void".to_string();
        }
        pub fn parse_bg(raw: &str) -> String {
            $(
                debug!("comparing {} and {}", raw, stringify!($name));
                if raw.eq_ignore_ascii_case(stringify!($name)) {
                    return format!("{}", Bg($name));
                }
            )*
            return "void".to_string();
        }
    }
}

make_parseurs! {
    Black,
    Blue,
    Cyan,
    Green,
    LightBlack,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightWhite,
    LightYellow,
    Magenta	,
    Red ,
    Reset,
    White,
    Yellow,
}

pub fn parse_entry(raw: &str) -> Result<String, ConfError> {
    let mut tokens = raw.split_whitespace();
    let fg = match tokens.next().map(|s| parse_fg(s)) {
        Some(c) => c,
        None => {
            return Err(ConfError::InvalidSkinEntry{reason:"Missing foreground in skin".to_string()});
        }
    };
    let bg = match tokens.next().map(|s| parse_bg(s)) {
        Some(c) => c,
        None => {
            return Err(ConfError::InvalidSkinEntry{reason:"Missing background in skin".to_string()});
        }
    };
    Ok(format!("{}{}", fg, bg))
}
