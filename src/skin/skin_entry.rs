//! Manage conversion of a user provided string
//! defining foreground and background colors into
//! a string with TTY colors

use {
    crate::errors::InvalidSkinError,
    serde::{de::Error, Deserialize, Deserializer},
    termimad::{
        CompoundStyle,
        parse_compound_style,
    },
};

/// Parsed content of a [skin] line of the conf.toml file
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
    /// Parse a string representation of a skin entry.
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
            .map(parse_compound_style)
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

