/// Manage conversion of a user provided string
/// defining foreground and background colors into
/// a string with TTY colors

use {
    super::*,
    crate::errors::InvalidSkinError,
    crossterm::style::{
        Attribute::{self, *},
        Attributes,
    },
    serde::{de::Error, Deserialize, Deserializer},
    termimad::CompoundStyle,
};

/// parsed content of a [skin] line of the conf.toml file
#[derive(Clone, Debug)]
pub struct SkinEntry {
    focused: CompoundStyle,
    unfocused: Option<CompoundStyle>,
}

impl SkinEntry {
    pub fn new(focused: CompoundStyle, unfocused: Option<CompoundStyle>) -> Self {
        Self { focused, unfocused }
    }
    pub fn get_focused(&self) -> &CompoundStyle {
        &self.focused
    }
    pub fn get_unfocused(&self) -> &CompoundStyle {
        self.unfocused.as_ref().unwrap_or(&self.focused)
    }
    /// parse a string representation of a skin entry.
    ///
    /// The general form is either "<focused>" or "<focused> / <unfocused>":
    /// It may be just the focused compound_style, or both
    /// the focused and the unfocused ones, in which case there's
    /// a '/' as separator.
    ///
    /// Each part is "<foreground color> <background color> <attributes>"
    /// where the attributes list may be empty.
    pub fn parse(s: &str) -> Result<Self, InvalidSkinError> {
        let mut parts = s.split('/');
        let focused = parse_compound_style(parts.next().unwrap())?;
        let unfocused = parts.next()
            .map(|p| parse_compound_style(p))
            .transpose()?;
        Ok(Self { focused, unfocused })
    }
}

impl<'de> Deserialize<'de> for SkinEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        SkinEntry::parse(&s)
            .map_err(|e| D::Error::custom(e.to_string()))
    }
}



fn parse_attribute(s: &str) -> Result<Attribute, InvalidSkinError> {
    match s {
        "bold" => Ok(Bold),
        "crossedout" => Ok(CrossedOut),
        "dim" => Ok(Dim), // does it do anything ?
        "italic" => Ok(Italic),
        "reverse" => Ok(Reverse),
        "underlined" => Ok(Underlined),
        "overlined" => Ok(OverLined),
        // following ones aren't supported by crossterm yet
        // "defaultforegroundcolor" => Ok(DefaultForegroundColor),
        // "defaultbackgroundcolor" => Ok(DefaultBackgroundColor),
        "slowblink" => Ok(SlowBlink),
        "rapidblink" => Ok(RapidBlink),
        _ => Err(InvalidSkinError::InvalidAttribute { raw: s.to_owned() }),
    }
}

/// parse a sequence of space separated style attributes
fn parse_attributes(s: &str) -> Result<Vec<Attribute>, InvalidSkinError> {
    s.split_whitespace().map(|t| parse_attribute(t)).collect()
}

fn parse_compound_style(s: &str) -> Result<CompoundStyle, InvalidSkinError> {
    let s = s.to_ascii_lowercase();
    let parts_rex = regex!(
        r"(?x)
        ^
        \s*
        (?P<fg>\w+(\([\d,\s]+\))?)
        \s+
        (?P<bg>\w+(\([\d,\s]+\))?)
        (?P<attributes>.*)
        \s*
        $
        "
    );
    if let Some(c) = parts_rex.captures(&s) {
        let fg_color = colors::parse(c.name("fg").unwrap().as_str())?;
        let bg_color = colors::parse(c.name("bg").unwrap().as_str())?;
        let attrs = parse_attributes(c.name("attributes").unwrap().as_str())?;
        Ok(CompoundStyle::new(
            fg_color,
            bg_color,
            Attributes::from(attrs.as_slice()),
        ))
    } else {
        Err(InvalidSkinError::InvalidStyle {
            style: s.to_owned(),
        })
    }
}
