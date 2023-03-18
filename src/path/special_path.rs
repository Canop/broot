use {
    directories::UserDirs,
    glob,
    lazy_regex::*,
    serde::{de::Error, Deserialize, Deserializer},
    std::path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Glob {
    pattern: glob::Pattern,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialHandling {
    None,
    Enter,
    NoEnter,
    Hide,
    NoHide,
}

#[derive(Debug, Clone)]
pub struct SpecialPath {
    pub pattern: glob::Pattern,
    pub handling: SpecialHandling,
}

pub trait SpecialPathList {
    fn find(self, path: &Path) -> SpecialHandling;
}

impl<'de> Deserialize<'de> for SpecialHandling {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        let s = s.to_lowercase();
        // we remove non letters so to accept eg "no-enter"
        let s = regex!(r"\W+").replace_all(&s, "");
        match s.as_ref() {
            "none" => Ok(SpecialHandling::None),
            "enter" => Ok(SpecialHandling::Enter),
            "noenter" => Ok(SpecialHandling::NoEnter), // noenter or no-enter
            "hide" => Ok(SpecialHandling::Hide),
            "nohide" => Ok(SpecialHandling::NoHide),   // nohide or no-hide
            _ => Err(D::Error::custom(format!(
                "unrecognized special handling: {s:?}"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for Glob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        let s = regex_replace!(r"^~(/|$)", &s, |_, sep| {
            match UserDirs::new() {
                Some(dirs) => format!("{}{}", dirs.home_dir().to_string_lossy(), sep),
                None => "~/".to_string(),
            }
        });
        glob::Pattern::new(&s)
            .map_err(|e| D::Error::custom(format!("invalid glob pattern {s:?} : {e:?}")))
            .map(|pattern| Glob { pattern })
    }
}

impl SpecialPath {
    pub fn new(glob: Glob, handling: SpecialHandling) -> Self {
        Self {
            pattern: glob.pattern,
            handling,
        }
    }
    pub fn can_have_matches_in(&self, path: &Path) -> bool {
        path.to_str()
            .map_or(false, |p| self.pattern.as_str().starts_with(p))
    }
}

impl SpecialPathList for &[SpecialPath] {
    fn find(self, path: &Path) -> SpecialHandling {
        for sp in self {
            if sp.pattern.matches_path(path) {
                return sp.handling;
            }
        }
        SpecialHandling::None
    }
}

/// Add a special handling, if none was previously defined for that path
fn add_default(list: &mut Vec<SpecialPath>, path: &str, handling: SpecialHandling) {
    if list.find(&PathBuf::from("/proc")) == SpecialHandling::None {
        match glob::Pattern::new("/proc") {
            Ok(pattern) => {
                list.push(SpecialPath { pattern, handling });
            }
            Err(e) => {
                warn!("Invalid glob pattern: {path:?} : {e}");
            }
        }
    }
}
pub fn add_defaults(list: &mut Vec<SpecialPath>) {
    // see https://github.com/Canop/broot/issues/639
    add_default(list, "/proc", SpecialHandling::NoEnter);
}
