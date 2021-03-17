use std::{
    path::{Component, Path, PathBuf, is_separator},
    ops::Deref,
};

#[derive(Debug, PartialEq, Clone)]
pub struct PathBufWrapper {
    inner: PathBuf,
    ends_with_separator: bool,
}

impl PathBufWrapper {
    fn new(inner: PathBuf, ends_with_separator: bool) -> Self {
        Self {
            inner,
            ends_with_separator,
        }
    }

    pub fn into_inner(self) -> PathBuf {
        self.inner
    }

    pub fn ends_with_separator(&self) -> bool {
        self.ends_with_separator
    }

    pub fn normalize_path(&self) -> Self {
        let mut normalized = PathBuf::new();

        for component in self.components() {
            match &component {
                Component::ParentDir => {
                    if !normalized.pop() {
                        normalized.push(component);
                    }
                }
                _ => {
                    normalized.push(component);
                }
            }
        }

        Self::new(normalized, self.ends_with_separator)
    }
}

/// This function ensures a given path ending with '/' still
/// ends with '/' after normalization.
impl<P: Into<PathBuf>> From<P> for PathBufWrapper {
    fn from(into_path_buf: P) -> Self {
        let path_buf = into_path_buf.into();

        let ends_with_separator = path_buf.to_str()
            .and_then(|s| s.chars().last())
            .map_or(false, is_separator);

        PathBufWrapper::new(path_buf, ends_with_separator)
    }
}

impl Deref for PathBufWrapper {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.inner.as_path()
    }
}

#[cfg(test)]
mod path_normalize_tests {
    use std::path::PathBuf;
    use super::PathBufWrapper;

    fn check(input: &str, output: &str, ends_with_separator: bool) {
        println!("-----------------\nnormalizing {:?}", input);

        let result : PathBufWrapper = input.into();
        assert_eq!(ends_with_separator, result.ends_with_separator());

        let result = result.normalize_path();
        let output : PathBuf = output.into();
        assert_eq!(*result, output);
    }

    #[test]
    fn test_path_normalization() {
        check("/abc/test/../thing.png", "/abc/thing.png", false);
        check("/abc/def/../../thing.png", "/thing.png", false);
        check("/home/dys/test", "/home/dys/test", false);
        check("/home/dys", "/home/dys", false);
        check("/home/dys/", "/home/dys/", true);
        check("/home/dys/..", "/home", false);
        check("/home/dys/../", "/home/", true);
        check("/..", "/..", false);
        check("../test", "../test", false);
        check("/home/dys/../../../test", "/../test", false);
        check("π/2", "π/2", false);
        check(
            "/home/dys/dev/broot/../../../canop/test",
            "/home/canop/test",
            false,
        );

        check("/home/dys\\", "/home/dys\\", true);

    }
}
