use crossterm::{
    Attribute::{self, *},
    Color::{self, *},
    ObjectStyle,
};
use regex::Regex;
/// Manage conversion of a user provided string
/// defining foreground and background colors into
/// a string with TTY colors
///
use std::result::Result;

use crate::errors::InvalidSkinError;
use crate::skin;

/// read a color from a string.
/// It may be either
/// - "none"
/// - one of the few known color name. Example: "darkred"
/// - grayscale with level in [0,24[. Example: "grey(5)"
/// - an Ansi code. Example "ansi(106)"
/// - RGB. Example: "rgb(25, 100, 0)"
/// This function needs a lowercase string (assuming lowercasing
/// has be done before, to ensure case-insensitive parsing)
fn parse_color(s: &str) -> Result<Option<Color>, InvalidSkinError> {

    lazy_static! {
        static ref ANSI_REX: Regex = Regex::new(r"^ansi\((?P<value>\d+)\)$").unwrap();
    }
    if let Some(c) = ANSI_REX.captures(&s) {
        let value: &str = c.name("value").unwrap().as_str();
        let value = value.parse();
        if let Ok(value) = value {
            return Ok(skin::ansi(value)); // all ANSI values are ok
        } else {
            return Err(InvalidSkinError::InvalidColor { raw: s.to_owned() });
        }
    }

    lazy_static! {
        static ref GRAY_REX: Regex = Regex::new(r"^gr[ae]y(?:scale)?\((?P<level>\d+)\)$").unwrap();
    }
    if let Some(c) = GRAY_REX.captures(&s) {
        let level: &str = c.name("level").unwrap().as_str();
        let level = level.parse();
        if let Ok(level) = level {
            if level > 23 {
                return Err(InvalidSkinError::InvalidGreyLevel { level });
            }
            return Ok(skin::gray(level));
        } else {
            return Err(InvalidSkinError::InvalidColor { raw: s.to_owned() });
        }
    }

    lazy_static! {
        static ref RGB_REX: Regex =
            Regex::new(r"^rgb\((?P<r>\d+),\s*(?P<g>\d+),\s*(?P<b>\d+)\)$").unwrap();
    }
    if let Some(c) = RGB_REX.captures(&s) {
        let r = c.name("r").unwrap().as_str().parse();
        let g = c.name("g").unwrap().as_str().parse();
        let b = c.name("b").unwrap().as_str().parse();
        if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
            return Ok(skin::rgb(r, g, b));
        } else {
            return Err(InvalidSkinError::InvalidColor { raw: s.to_owned() });
        }
    }

    match s.as_ref() {
        // TODO: we could add a table of common colors and map to ansi colors
        "black" => Ok(skin::rgb(0, 0, 0)), // crossterm black isn't black
        "blue" => Ok(Some(Blue)),
        "cyan" => Ok(Some(Cyan)),
        "darkblue" => Ok(Some(DarkBlue)),
        "darkcyan" => Ok(Some(DarkCyan)),
        "darkgreen" => Ok(Some(DarkGreen)),
        "darkmagenta" => Ok(Some(DarkMagenta)),
        "darkred" => Ok(Some(DarkRed)),
        "green" => Ok(Some(Green)),
        "grey" => Ok(Some(Grey)),
        "magenta" => Ok(Some(Magenta)),
        "red" => Ok(Some(Red)),
        "yellow" => Ok(Some(Yellow)),
        "darkyellow" => Ok(Some(DarkYellow)),
        "white" => Ok(Some(White)),
        "none" => Ok(None),
        _ => Err(InvalidSkinError::InvalidColor { raw: s.to_owned() }),
    }
}

///
fn parse_attribute(s: &str) -> Result<Attribute, InvalidSkinError> {
    match s {
        "bold" => Ok(Bold),
        "crossedout" => Ok(CrossedOut),
        "dim" => Ok(Dim), // does it do anything ?
        "italic" => Ok(Italic),
        "underlined" => Ok(Underlined),
        "overlined" => Ok(OverLined),
        _ => Err(InvalidSkinError::InvalidAttribute { raw: s.to_owned() }),
    }
}

/// parse a sequence of space separated style attributes
fn parse_attributes(s: &str) -> Result<Vec<Attribute>, InvalidSkinError> {
    s.split_whitespace().map(|t| parse_attribute(t)).collect()
}

pub fn parse_object_style(s: &str) -> Result<ObjectStyle, InvalidSkinError> {
    let s = s.to_ascii_lowercase();
    lazy_static! {
        static ref PARTS_REX: Regex = Regex::new(
            r"(?x)
            ^
            (?P<fg>\w+(\([\d,\s]+\))?)
            \s+
            (?P<bg>\w+(\([\d,\s]+\))?)
            (?P<attributes>.*)
            $
            "
        )
        .unwrap();
    }
    if let Some(c) = PARTS_REX.captures(&s) {
        debug!("match for {:?}", s);
        let fg_color = parse_color(c.name("fg").unwrap().as_str())?;
        let bg_color = parse_color(c.name("bg").unwrap().as_str())?;
        let attrs = parse_attributes(c.name("attributes").unwrap().as_str())?;
        Ok(ObjectStyle {
            fg_color,
            bg_color,
            attrs,
        })
    } else {
        debug!("NO match for {:?}", s);
        Err(InvalidSkinError::InvalidStyle {
            style: s.to_owned(),
        })
    }
}
