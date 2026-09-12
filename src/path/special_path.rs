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
    pub fn find<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> SpecialHandling {
        self.entries
            .iter()
            .find(|sp| sp.pattern.matches_path(path.as_ref()))
            .map(|sp| sp.handling)
            .unwrap_or_default()
    }
    pub fn show(
        &self,
        path: &Path,
    ) -> Directive {
        self.find(path).show
    }
    pub fn list(
        &self,
        path: &Path,
    ) -> Directive {
        self.find(path).list
    }
    pub fn sum(
        &self,
        path: &Path,
    ) -> Directive {
        self.find(path).sum
    }
    /// Add a special handling, if none was previously defined for that path
    pub fn add_default(
        &mut self,
        path: &str,
        handling: SpecialHandling,
    ) {
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
        self.add_default(
            "/proc",
            SpecialHandling {
                show: Directive::Default,
                list: Directive::Never,
                sum: Directive::Never,
            },
        );
    }
    /// Return a potentially smaller set of special paths, reduced
    /// to what can be in path
    pub fn reduce(
        &self,
        path: &Path,
    ) -> Self {
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
    pub fn new(
        pattern: glob::Pattern,
        handling: SpecialHandling,
    ) -> Self {
        Self { pattern, handling }
    }
    /// Tell whether the pattern may match some path inside the given
    /// directory. Used to reduce the set of special paths to check
    /// during a recursive computation.
    ///
    /// When in doubt, this returns true: a false positive only costs a
    /// few glob matchings, while a false negative would make broot
    /// ignore the special handling of a path.
    pub fn can_have_matches_in(
        &self,
        path: &Path,
    ) -> bool {
        let Some(p) = path.to_str() else {
            return true;
        };
        let pattern = self.pattern.as_str();
        // every path matching the pattern starts with the part of the
        // pattern preceding the first glob metacharacter
        let fixed_len = pattern.find(['*', '?', '[']).unwrap_or(pattern.len());
        let fixed = &pattern[..fixed_len];
        if fixed.len() >= p.len() {
            fixed.starts_with(p)
        } else {
            // the pattern must have a wildcard part (otherwise it only matches
            // a path shorter than `p`, thus not inside it) and its fixed part
            // must be compatible with `p`
            fixed_len < pattern.len() && p.starts_with(fixed)
        }
    }
}

#[cfg(test)]
mod special_path_tests {
    use super::*;

    fn can_have_matches_in(
        pattern: &str,
        dir: &str,
    ) -> bool {
        SpecialPath::new(
            glob::Pattern::new(pattern).unwrap(),
            SpecialHandling::default(),
        )
        .can_have_matches_in(Path::new(dir))
    }

    #[test]
    fn test_can_have_matches_in() {
        // a relative conf entry is globbed as "**/<name>" and can match at any depth
        assert!(can_have_matches_in("**/node_modules", "/home/dys/dev"));
        assert!(can_have_matches_in("**/node_modules", "/"));
        // an absolute pattern only matches in its own branch
        assert!(can_have_matches_in("/media", "/"));
        assert!(!can_have_matches_in("/media", "/home"));
        assert!(can_have_matches_in("/home/dys/.cargo", "/home/dys"));
        assert!(!can_have_matches_in("/home/dys/.cargo", "/home/other"));
        // an absolute pattern with a wildcard can match deeper
        assert!(can_have_matches_in("/home/*/.cargo", "/home/dys"));
        assert!(!can_have_matches_in("/home/*/.cargo", "/var/log"));
    }
}
