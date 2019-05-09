
// This file is ignored for now. I'll have to rewrite
// skin configuration parsing for crossterm when the
// rest is proven OK

use crate::errors::{ConfError, InvalidSkinError};
use regex::Regex;
/// Manage conversion of a user provided string
/// defining foreground and background colors into
/// a string with TTY colors
///
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
struct TypedColor {color: Box<Color>, typ: ColorType}

impl std::fmt::Display for TypedColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.typ {
            ColorType::Foreground => self.color.write_fg(f),
            ColorType::Background => self.color.write_bg(f),
        }
    }
}

macro_rules! define_color_from_name {
    ( $($name:ident,)* ) => {
        fn color_from_name(raw: &str) -> Result<Box<Color>, InvalidSkinError> {
            if raw.eq_ignore_ascii_case("none") {
                Ok(Box::new(Reset))
            } $(else if raw.eq_ignore_ascii_case(stringify!($name)) {
                Ok(Box::new($name))
            })* else if let Some(level) = parse_gray(raw)? {
                Ok(Box::new(AnsiValue::grayscale(level)))
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
    Yellow,
}

pub fn parse_config_entry(key: &str, value: &str) -> Result<String, ConfError>{
    match key {
        k if k.ends_with("_fg") => Ok(ColorType:: Foreground),
        k if k.ends_with("_bg") => Ok(ColorType:: Background),
        _ => Err(InvalidSkinError::BadKey)
    }.and_then(|typ|
        Ok(TypedColor{ color: color_from_name(value)?, typ }.to_string())
    ).map_err(|source| ConfError::InvalidSkinEntry{key: key.into(), source})
}
