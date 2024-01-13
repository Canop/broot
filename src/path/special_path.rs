use {
    glob,
    serde::Deserialize,
    std::path::Path,
};

///// Wrap a glob pattern to add the Deserialize trait
//#[derive(Debug, Clone, PartialEq, Hash, Eq)]
//pub struct Glob {
//    pattern: glob::Pattern,
//}

#[derive(Clone, Copy, Debug, Deserialize, Default, PartialEq)]
pub struct SpecialHandling {
    #[serde(default)]
    pub show: Directive,
    #[serde(default)]
    pub list: Directive,
    #[serde(default)]
    pub sum: Directive,
}

#[derive(Clone, Debug, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Directive {
    #[default]
    Default,
    Never,
    Always,
}

#[derive(Debug, Clone)]
pub struct SpecialPath {
    pub pattern: glob::Pattern,
    pub handling: SpecialHandling,
}

#[derive(Debug, Clone)]
pub struct SpecialPaths {
    pub entries: Vec<SpecialPath>,
}

impl SpecialPaths {
    pub fn find<P: AsRef<Path>>(&self, path: P) -> SpecialHandling {
        self
            .entries.iter()
            .find(|sp| sp.pattern.matches_path(path.as_ref()))
            .map(|sp| sp.handling)
            .unwrap_or_default()
    }
    pub fn show(&self, path: &Path) -> Directive {
        self.find(path).show
    }
    pub fn list(&self, path: &Path) -> Directive {
        self.find(path).list
    }
    pub fn sum(&self, path: &Path) -> Directive {
        self.find(path).sum
    }
    /// Add a special handling, if none was previously defined for that path
    pub fn add_default(&mut self, path: &str, handling: SpecialHandling) {
        if self.find(path) != Default::default() {
            return;
        }
        match glob::Pattern::new("/proc") {
            Ok(pattern) => {
                self.entries.push(SpecialPath { pattern, handling });
            }
            Err(e) => {
                warn!("Invalid glob pattern: {path:?} : {e}");
            }
        }
    }
    pub fn add_defaults(&mut self) {
        // see https://github.com/Canop/broot/issues/639
        self.add_default("/proc", SpecialHandling {
            show: Directive::Default,
            list: Directive::Never,
            sum: Directive::Never,
        });
    }
    /// Return a potentially smaller set of special paths, reduced
    /// to what can be in path
    pub fn reduce(&self, path: &Path) -> Self {
        let entries = self
            .entries
            .iter()
            .filter(|sp| sp.can_have_matches_in(path))
            .cloned()
            .collect();
        Self { entries }
    }
}

impl SpecialPath {
    pub fn new(pattern: glob::Pattern, handling: SpecialHandling) -> Self {
        Self {
            pattern,
            handling,
        }
    }
    pub fn can_have_matches_in(&self, path: &Path) -> bool {
        path.to_str()
            .map_or(false, |p| self.pattern.as_str().starts_with(p))
    }
}


