use {
    crate::{
        errors::ConfError,
        path::*,
    },
    directories::UserDirs,
    lazy_regex::*,
    serde::Deserialize,
    std::collections::HashMap,
};


type SpecialPathsConf = HashMap<GlobConf, SpecialHandlingConf>;

#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Eq)]
#[serde(transparent)]
pub struct GlobConf {
    pub pattern: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(untagged)]
pub enum SpecialHandlingConf {
    Shortcut(SpecialHandlingShortcut),
    Detailed(SpecialHandling),
}

#[derive(Clone, Debug, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpecialHandlingShortcut {
    None,
    Enter,
    #[serde(alias = "no-enter")]
    NoEnter,
    Hide,
    #[serde(alias = "no-hide")]
    NoHide,
}

impl From<SpecialHandlingShortcut> for SpecialHandling {
    fn from(shortcut: SpecialHandlingShortcut) -> Self {
        use Directive::*;
        match shortcut {
            SpecialHandlingShortcut::None => SpecialHandling {
                show: Default, list: Default, sum: Default,
            },
            SpecialHandlingShortcut::Enter => SpecialHandling {
                show: Always, list: Always, sum: Always,
            },
            SpecialHandlingShortcut::NoEnter => SpecialHandling {
                show: Default, list: Never, sum: Never,
            },
            SpecialHandlingShortcut::Hide => SpecialHandling {
                show: Never, list: Default, sum: Never,
            },
            SpecialHandlingShortcut::NoHide => SpecialHandling {
                show: Always, list: Default, sum: Default,
            },
        }
    }
}

impl From<SpecialHandlingConf> for SpecialHandling {
    fn from(conf: SpecialHandlingConf) -> Self {
        match conf {
            SpecialHandlingConf::Shortcut(shortcut) => shortcut.into(),
            SpecialHandlingConf::Detailed(handling) => handling,
        }
    }
}

impl TryFrom<&SpecialPathsConf> for SpecialPaths {
    type Error = ConfError;
    fn try_from(map: &SpecialPathsConf) -> Result<Self, ConfError> {
        let mut entries = Vec::new();
        for (k, v) in map {
            entries.push(SpecialPath::new(k.to_glob()?, (*v).into()));
        }
        Ok(Self { entries })
    }
}

impl GlobConf {
    pub fn to_glob(&self) -> Result<glob::Pattern, ConfError> {
        let s = regex_replace!(r"^~(/|$)", &self.pattern, |_, sep| {
            match UserDirs::new() {
                Some(dirs) => format!("{}{}", dirs.home_dir().to_string_lossy(), sep),
                None => "~/".to_string(),
            }
        });
        let glob = if s.starts_with('/') || s.starts_with('~') {
            glob::Pattern::new(&s)
        } else {
            let pattern = format!("**/{}", &s);
            glob::Pattern::new(&pattern)
        };
        glob.map_err(|_| ConfError::InvalidGlobPattern {
            pattern: self.pattern.to_string(),
        })
    }
}
