/// Manage conversion of a user provided string
/// defining foreground and background colors into
/// a string with TTY colors
///
/// The code is ugly. Really. I know.
/// I found no clean way to deal with termion colors.
use std::result::Result;
use regex::Regex;
use termion::color::*;
use crate::errors::ConfError;

fn parse_gray(raw: &str) -> Result<Option<u8>, ConfError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"^grayscale\((?P<level>\d+)\)$"
        )
        .unwrap();
    }
    if let Some(c) = RE.captures(raw) {
        if let Some(level) = c.name("level") {
            if let Ok(level) = level.as_str().parse() {
                return if level < 24 {
                    Ok(Some(level))
                } else {
                    Err(ConfError::InvalidSkinEntry{
                        reason: "gray level must be between 0 and 23".to_string()
                    })
                }
            }
        }
    }
    Ok(None)
}

macro_rules! make_parseurs {
    (
        $($name:tt,)*
    ) => {
        pub fn parse_fg(raw: &str) -> Result<String, ConfError> {
            if raw.eq_ignore_ascii_case("none") {
                return Ok(format!("{}", Fg(Reset)));
            }
            $(
                if raw.eq_ignore_ascii_case(stringify!($name)) {
                    return Ok(format!("{}", Fg($name)));
                }
            )*
            if let Some(level) = parse_gray(raw)? {
                return Ok(format!("{}", Fg(AnsiValue::grayscale(level))));
            }
            return Err(ConfError::InvalidSkinEntry{reason:raw.to_string()});
        }
        pub fn parse_bg(raw: &str) -> Result<String, ConfError> {
            if raw.eq_ignore_ascii_case("none") {
                return Ok(format!("{}", Bg(Reset)));
            }
            $(
                debug!("comparing {} and {}", raw, stringify!($name));
                if raw.eq_ignore_ascii_case(stringify!($name)) {
                    debug!(" -> ok");
                    return Ok(format!("{}", Bg($name)));
                } else {
                    debug!(" -> not ok");
                }
            )*
            if let Some(level) = parse_gray(raw)? {
                return Ok(format!("{}", Bg(AnsiValue::grayscale(level))));
            }
            return Err(ConfError::InvalidSkinEntry{reason:raw.to_string()});
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
    Magenta,
    Red,
    Reset,
    White,
    Yellow,
}

