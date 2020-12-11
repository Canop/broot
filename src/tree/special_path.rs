use {
    glob,
    serde::{de::Error, Deserialize, Deserializer},
    std::path::Path,
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
}

#[derive(Debug, Clone)]
pub struct SpecialPath {
    pub pattern: glob::Pattern,
    pub handling: SpecialHandling,
}

pub trait SpecialPathList {
    fn find(
        self,
        path: &Path,
    ) -> SpecialHandling;
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
            "noenter" => Ok(SpecialHandling::NoEnter),
            "hide" => Ok(SpecialHandling::Hide),
            _ => Err(D::Error::custom(
                format!("unrecognized special handling: {:?}", s)
            )),
        }

    }
}

impl<'de> Deserialize<'de> for Glob {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
    let s = String::deserialize(deserializer)?;
    glob::Pattern::new(&s)
        .map_err(|e| D::Error::custom(format!("invalid glob pattern {:?} : {:?}", s, e)))
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
}

impl SpecialPathList for &[SpecialPath] {
    fn find(
        self,
        path: &Path,
    ) -> SpecialHandling {
        for sp in self {
            if sp.pattern.matches_path(path) {
                return sp.handling;
            }
        }
        SpecialHandling::None
    }
}
