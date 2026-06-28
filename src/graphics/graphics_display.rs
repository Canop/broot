use {
    cli_log::*,
    serde::Deserialize,
};

/// Which terminal-graphics protocol broot uses for high-resolution image
/// previews. This is the protocol selector; `kitty-graphics-display` only tunes
/// the Kitty display mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphicsDisplay {
    /// no terminal graphics (text / half-block fallback only)
    None,
    /// detect: Kitty when available, else Sixel
    #[default]
    Auto,
    /// force the Kitty graphics protocol
    Kitty,
    /// force the Sixel graphics protocol
    Sixel,
}

impl GraphicsDisplay {
    /// Parse a `BROOT_GRAPHICS_PROTOCOL` value (case-insensitive, trimmed).
    /// Unknown -> `None`.
    fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "none" => Some(Self::None),
            "auto" => Some(Self::Auto),
            "kitty" => Some(Self::Kitty),
            "sixel" => Some(Self::Sixel),
            _ => None,
        }
    }
}

/// The `BROOT_GRAPHICS_PROTOCOL` env override, if set and recognized.
/// An unrecognized value is logged and ignored (config then applies).
pub fn graphics_display_from_env() -> Option<GraphicsDisplay> {
    let raw = std::env::var("BROOT_GRAPHICS_PROTOCOL").ok()?;
    match GraphicsDisplay::parse(&raw) {
        Some(gd) => Some(gd),
        None => {
            warn!("ignoring unrecognized BROOT_GRAPHICS_PROTOCOL={raw:?}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(serde::Deserialize)]
    struct Wrap {
        d: GraphicsDisplay,
    }

    #[test]
    fn default_is_auto() {
        assert_eq!(GraphicsDisplay::default(), GraphicsDisplay::Auto);
    }

    #[test]
    fn deserializes_snake_case() {
        let p = |s: &str| toml::from_str::<Wrap>(s).unwrap().d;
        assert_eq!(p("d = \"none\""), GraphicsDisplay::None);
        assert_eq!(p("d = \"auto\""), GraphicsDisplay::Auto);
        assert_eq!(p("d = \"kitty\""), GraphicsDisplay::Kitty);
        assert_eq!(p("d = \"sixel\""), GraphicsDisplay::Sixel);
    }

    #[test]
    fn parse_maps_known_values_case_insensitively() {
        assert_eq!(GraphicsDisplay::parse("none"), Some(GraphicsDisplay::None));
        assert_eq!(GraphicsDisplay::parse("AUTO"), Some(GraphicsDisplay::Auto));
        assert_eq!(GraphicsDisplay::parse(" Kitty "), Some(GraphicsDisplay::Kitty));
        assert_eq!(GraphicsDisplay::parse("sixel"), Some(GraphicsDisplay::Sixel));
    }

    #[test]
    fn parse_rejects_unknown() {
        assert_eq!(GraphicsDisplay::parse("bogus"), None);
        assert_eq!(GraphicsDisplay::parse(""), None);
    }
}
