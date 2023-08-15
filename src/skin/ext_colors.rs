use {
    crate::{
        errors::InvalidSkinError,
    },
    ahash::AHashMap,
    crokey::crossterm::style::Color,
    lazy_regex::*,
    std::convert::TryFrom,
    termimad::parse_color,
};


/// a map from file extension to the foreground
/// color to use when drawing the tree
#[derive(Debug, Clone, Default)]
pub struct ExtColorMap {
    map: AHashMap<String, Color>,
}

impl ExtColorMap {
    /// return the color to use, or None when the default color
    /// of files should apply
    pub fn get(&self, ext: &str) -> Option<Color> {
        self.map.get(ext).copied()
    }
    pub fn set(&mut self, ext: String, raw_color: &str) -> Result<(), InvalidSkinError> {
        if !regex_is_match!("^none$"i, raw_color) {
            let color = parse_color(raw_color)?;
            self.map.insert(ext, color);
        }
        Ok(())
    }
}

impl TryFrom<&AHashMap<String, String>> for ExtColorMap {
    type Error = InvalidSkinError;
    fn try_from(raw_map: &AHashMap<String, String>) -> Result<Self, Self::Error> {
        let mut map = ExtColorMap::default();
        for (k, v) in raw_map {
            map.set(k.to_string(), v)?;
        }
        Ok(map)
    }
}

