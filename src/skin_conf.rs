
// This file is ignored for now. I'll have to rewrite
// skin configuration parsing for crossterm when the
// rest is proven OK

use regex::Regex;
/// Manage conversion of a user provided string
/// defining foreground and background colors into
/// a string with TTY colors
///
use std::result::Result;
use crossterm::{Attribute::{self, *}, Color::{self, *}, ObjectStyle};

use crate::errors::InvalidSkinError;
use crate::skin;

fn parse_color(s: &str) -> Result<Option<Color>, InvalidSkinError> {
    let s = s.to_ascii_lowercase();

    lazy_static! {
        static ref GRAY_REX: Regex = Regex::new(r"^gr[ae]y(?:scale)?\((?P<level>\d+)\)$").unwrap();
    }
    if let Some(c) = GRAY_REX.captures(&s) {
        let level: &str = c.name("level").unwrap().as_str();
        let level = level.parse();
        if let Ok(level) = level {
            if level > 23 {
                return Err( InvalidSkinError::InvalidGreyLevel { level });
            }
            return Ok(skin::gray(level));
        } else {
            return Err( InvalidSkinError::InvalidColor{raw: s.to_owned()});
        }
    }

    lazy_static! {
        static ref RGB_REX: Regex = Regex::new(r"^rgb\((?P<r>\d+),\s*(?P<g>\d+),\s*(?P<b>\d+)\)$").unwrap();
    }
    if let Some(c) = RGB_REX.captures(&s) {
        let r = c.name("r").unwrap().as_str().parse();
        let g = c.name("g").unwrap().as_str().parse();
        let b = c.name("b").unwrap().as_str().parse();
        if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
            return Ok(skin::rgb(r, g, b));
        } else {
            return Err( InvalidSkinError::InvalidColor{raw: s.to_owned()});
        }
    }

    match s.as_ref() {
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
        "none" => Ok(None),
        _ => Err(InvalidSkinError::InvalidColor{raw: s.to_owned()}),
    }
}

fn parse_attributes(s: &str) -> Result<Vec<Attribute>, InvalidSkinError> {
    Ok(Vec::new())
}

pub fn parse_object_style(s: &str) -> Result<ObjectStyle, InvalidSkinError>{
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
    if let Some(c) = PARTS_REX.captures(s) {
        debug!("match for {:?}", s);
        let fg_color = parse_color(c.name("fg").unwrap().as_str())?;
        let bg_color = parse_color(c.name("bg").unwrap().as_str())?;
        let attrs = parse_attributes(c.name("attributes").unwrap().as_str())?;
        Ok(ObjectStyle { fg_color, bg_color, attrs })
    } else {
        debug!("NO match for {:?}", s);
        Err(InvalidSkinError::InvalidStyle{style: s.to_owned()})
    }
}

