use std::result::Result;
use std::ascii::AsciiExt;
use regex::Regex;
use termion::color::{self, *};
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
            $(
                debug!("comparing {} and {}", raw, stringify!($name));
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
            $(
                debug!("comparing {} and {}", raw, stringify!($name));
                if raw.eq_ignore_ascii_case(stringify!($name)) {
                    return Ok(format!("{}", Bg($name)));
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

pub fn parse_entry(raw: &str) -> Result<String, ConfError> {
    let mut tokens = raw.split_whitespace();
    let fg = match tokens.next() {
        Some(c) => parse_fg(c)?,
        None => {
            return Err(ConfError::InvalidSkinEntry{reason:"Missing foreground in skin".to_string()});
        }
    };
    let bg = match tokens.next() {
        Some(c) => parse_bg(c)?,
        None => {
            return Err(ConfError::InvalidSkinEntry{reason:"Missing background in skin".to_string()});
        }
    };
    Ok(format!("{}{}", fg, bg))
}
