use {
    super::*,
    crate::errors::InvalidSkinError,
    crossterm::style::{
        Color::{self, *},
    },
    lazy_regex::regex_captures,
};

/// read a color from a string.
/// It may be either
/// - "none"
/// - one of the few known color name. Example: "darkred"
/// - grayscale with level in [0,24[. Example: "grey(5)"
/// - an Ansi code. Example "ansi(106)"
/// - RGB. Example: "rgb(25, 100, 0)"
/// This function needs a lowercase string (assuming lowercasing
/// has be done before, to ensure case-insensitive parsing)
pub fn parse(s: &str) -> Result<Option<Color>, InvalidSkinError> {
    if let Some((_, value)) = regex_captures!(r"^ansi\((?P<value>\d+)\)$", &s) {
        let value = value.parse();
        if let Ok(value) = value {
            return Ok(ansi(value)); // all ANSI values are ok
        } else {
            return Err(InvalidSkinError::InvalidColor { raw: s.to_owned() });
        }
    }

    if let Some((_, level)) = regex_captures!(r"^gr[ae]y(?:scale)?\((?P<level>\d+)\)$", &s) {
        let level = level.parse();
        if let Ok(level) = level {
            if level > 23 {
                return Err(InvalidSkinError::InvalidGreyLevel { level });
            }
            return Ok(gray(level));
        } else {
            return Err(InvalidSkinError::InvalidColor { raw: s.to_owned() });
        }
    }

    if let Some((_, r, g, b)) = regex_captures!(r"^rgb\((?P<r>\d+),\s*(?P<g>\d+),\s*(?P<b>\d+)\)$", &s) {
        if let (Ok(r), Ok(g), Ok(b)) = (r.parse(), g.parse(), b.parse()) {
            return Ok(rgb(r, g, b));
        } else {
            return Err(InvalidSkinError::InvalidColor { raw: s.to_owned() });
        }
    }

    match s {
        "black" => Ok(Some(AnsiValue(16))),
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
        "white" => Ok(Some(AnsiValue(231))),
        "none" => Ok(None),
        _ => Err(InvalidSkinError::InvalidColor { raw: s.to_owned() }),
    }
}

