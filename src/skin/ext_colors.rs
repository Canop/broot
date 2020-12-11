use {
    super::colors,
    crate::{
        errors::InvalidSkinError,
    },
    crossterm::style::Color,
    fnv::FnvHashMap,
    std::convert::TryFrom,
};


/// a map from file extension to the foreground
/// color to use when drawing the tree
#[derive(Debug, Clone, Default)]
pub struct ExtColorMap {
    map: FnvHashMap<String, Color>,
}

impl ExtColorMap {
    /// return the color to use, or None when the default color
    /// of files should apply
    pub fn get(&self, ext: &str) -> Option<Color> {
        self.map.get(ext).copied()
    }
    pub fn set(&mut self, ext: String, raw_color: &str) -> Result<(), InvalidSkinError> {
        if let Some(color) = colors::parse(raw_color)? {
            self.map.insert(ext, color);
        }
        Ok(())
    }
}

impl TryFrom<&FnvHashMap<String, String>> for ExtColorMap {
    type Error = InvalidSkinError;
    fn try_from(raw_map: &FnvHashMap<String, String>) -> Result<Self, Self::Error> {
        let mut map = ExtColorMap::default();
        for (k, v) in raw_map {
            map.set(k.to_string(), v)?;
        }
        Ok(map)
    }
}

