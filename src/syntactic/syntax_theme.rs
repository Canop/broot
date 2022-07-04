//! The supported syntax themes coming from syntect.
//!
//! This enumeration may change but right now the values are the ones from
//!  https://docs.rs/syntect/latest/syntect/highlighting/struct.ThemeSet.html

use {
    crate::{
        errors::ConfError,
    },
    serde::{Deserialize, Deserializer},
    std::str::FromStr,
};

macro_rules! Themes {
    (
        $($enum_name:ident: $syntect_name: literal,)*
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum SyntaxTheme {
            $($enum_name,)*
        }
        impl FromStr for SyntaxTheme {
            type Err = ConfError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use crate::syntactic::SyntaxTheme::*;
                let s = s.to_lowercase();
                $(
                    if s == stringify!($enum_name).to_lowercase() {
                        return Ok($enum_name);
                    }
                    if s == $syntect_name.to_lowercase() {
                        return Ok($enum_name);
                    }
                )*
                Err(ConfError::InvalidSyntaxTheme { name: s.to_string() })
            }
        }
        impl SyntaxTheme {
            pub fn name(self) -> &'static str {
                use crate::syntactic::SyntaxTheme::*;
                match self {
                    $($enum_name => stringify!($enum_name),)*
                }
            }
            pub fn syntect_name(self) -> &'static str {
                use crate::syntactic::SyntaxTheme::*;
                match self {
                    $($enum_name => $syntect_name,)*
                }
            }
        }
        impl Default for SyntaxTheme {
            fn default() -> Self {
                Self::MochaDark
            }
        }
        pub static SYNTAX_THEMES: &[SyntaxTheme] = &[
            $(crate::syntactic::SyntaxTheme::$enum_name,)*
        ];
    }
}

Themes! {
    GitHub: "InspiredGitHub",
    SolarizedDark: "Solarized (dark)",
    SolarizedLight: "Solarized (light)",
    EightiesDark: "base16-eighties.dark",
    MochaDark: "base16-mocha.dark",
    OceanDark: "base16-ocean.dark",
    OceanLight: "base16-ocean.light",
}

impl<'de> Deserialize<'de> for SyntaxTheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

