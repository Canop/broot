use crate::errors::{ConfError, InvalidSkinError};
use regex::Regex;
/// Manage conversion of a user provided string
/// defining foreground and background colors into
/// a string with TTY colors
///
/// The code is ugly. Really. I know.
/// I found no clean way to deal with termion colors.
use std::result::Result;
use termion::color::*;

fn parse_gray_option(raw: &str) -> Option<u8> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^grayscale\((?P<level>\d+)\)$").unwrap();
    }
    RE.captures(raw)?.name("level")?.as_str().parse().ok()
}

fn parse_gray(raw: &str) -> Result<Option<u8>, InvalidSkinError> {
    match parse_gray_option(raw) {
        Some(level) if level < 24 => Ok(Some(level)),
        Some(level) => Err(InvalidSkinError::InvalidGreyLevel { level }),
        None => Ok(None),
    }
}

enum ColorType {Foreground, Background}

fn color_str<C: Color>(color: C, typ: ColorType) -> String {
    match typ {
        ColorType::Foreground => format!("{}", Fg(color)),
        ColorType::Background => format!("{}", Bg(color)),
    }
}

macro_rules! define_color_from_name {
    (
        $($name:ident),*
    ) => {
        fn color_from_name(raw: &str, typ: ColorType) -> Result<String, InvalidSkinError> {
            if raw.eq_ignore_ascii_case("none") {
                Ok(color_str(Reset, typ))
            }
            $(
                else if raw.eq_ignore_ascii_case(stringify!($name)) {
                    Ok(color_str($name, typ))
                }
            )*
            else if let Some(level) = parse_gray(raw)? {
                return Ok(color_str(AnsiValue::grayscale(level), typ));
            } else {
                Err(InvalidSkinError::InvalidColor{ raw: raw.to_string() })
            }
        }
    }
}

define_color_from_name! {
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
    Magenta,
    Red,
    Reset,
    White,
    Yellow
}

pub fn parse_config_entry(key: &str, value: &str) -> Result<String, ConfError>{
    match key {
        k if k.ends_with("_fg") => Ok(ColorType:: Foreground),
        k if k.ends_with("_bg") => Ok(ColorType:: Background),
        _ => Err(InvalidSkinError::BadKey)
    }.and_then(|typ| color_from_name(value, typ))
    .map_err(|source| ConfError::InvalidSkinEntry{key: key.into(), source})
}
