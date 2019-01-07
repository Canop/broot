#![warn(clippy::all)]

// implements parsing and applying .gitignore files
// Also manages a stack of such files, because more than one
// can apply for a dir (i.e when entering a directory we
// may add a gitignore file to the stack
use glob;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::{Path, PathBuf};

// a simple rule of a gitignore file
#[derive(Clone)]
struct GitIgnoreRule {
    ok: bool,        // does this rule when matched means the file is good? (usually false)
    directory: bool, // whether this rule only applies to directories
    filename: bool,  // does this rule apply to just the filename
    pattern: glob::Pattern,
    pattern_options: glob::MatchOptions,
}

impl GitIgnoreRule {
    fn from(line: &str, dir: &Path) -> Option<GitIgnoreRule> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                ^\s*
                (!)?    # 1 : negation
                (.+?)   # 2 : pattern
                (/)?    # 3 : directory
                \s*$
                "
            )
            .unwrap();
        }
        if line.starts_with('#') {
            return None; // comment line
        }
        if let Some(c) = RE.captures(line) {
            if let Some(p) = c.get(2) {
                let mut p = p.as_str().to_string();
                let has_separator = p.contains('/');
                if has_separator && p.starts_with('/') {
                    p = dir.to_string_lossy().to_string() + &p;
                }
                if let Ok(pattern) = glob::Pattern::new(&p) {
                    let pattern_options = glob::MatchOptions {
                        case_sensitive: true, // not really sure about this one
                        require_literal_leading_dot: false,
                        require_literal_separator: has_separator,
                    };
                    return Some(GitIgnoreRule {
                        ok: c.get(1).is_some(), // if negation
                        pattern,
                        directory: c.get(3).is_some(),
                        filename: !has_separator,
                        pattern_options,
                    });
                }
            }
        }
        None
    }
}

// the rules of a gitignore file
#[derive(Clone)]
pub struct GitIgnoreFile {
    rules: Vec<GitIgnoreRule>,
}
impl GitIgnoreFile {
    pub fn new(path: &Path) -> Result<GitIgnoreFile> {
        let f = File::open(path)?;
        let parent = path.parent().unwrap();
        let mut rules: Vec<GitIgnoreRule> = Vec::new();
        for line in BufReader::new(f).lines() {
            if let Some(rule) = GitIgnoreRule::from(&line?, &parent) {
                rules.push(rule);
            }
        }
        // the last rule applicable to a path is the right one. So
        // we reverse the list to easily iterate from the last one to the first one
        rules.reverse();
        debug!(
            "loaded .gitignore file {:?} with {} rules",
            path,
            rules.len()
        );
        Ok(GitIgnoreFile { rules })
    }
}

// a stack of the gitignore files applying to a directory
pub struct GitIgnoreFilter {
    pub files: Vec<GitIgnoreFile>,
}
impl GitIgnoreFilter {
    pub fn applicable_to(path: &Path) -> GitIgnoreFilter {
        let mut filter = GitIgnoreFilter { files: Vec::new() };
        for ignore_file in GitIgnoreFilesFinder::for_dir(path) {
            match GitIgnoreFile::new(&ignore_file) {
                Ok(gif) => {
                    filter.files.push(gif);
                }
                Err(e) => {
                    info!("reading GIF failed: {:?}", e);
                }
            }
        }
        filter
    }
    pub fn extended_to(&self, dir: &Path) -> GitIgnoreFilter {
        let mut files = self.files.clone();
        let ignore_file = dir.join(".gitignore");
        if let Ok(gif) = GitIgnoreFile::new(&ignore_file) {
            files.push(gif);
        }
        GitIgnoreFilter { files }
    }
    pub fn accepts(&self, path: &Path, filename: &str, directory: bool) -> bool {
        for file in &self.files {
            for rule in &file.rules {
                if rule.directory && !directory {
                    continue;
                }
                if rule.filename {
                    if rule.pattern.matches_with(filename, &rule.pattern_options) {
                        //debug!("rule matches filename {:?} -> ok={}", path, rule.ok);
                        return rule.ok;
                    }
                } else if rule.pattern.matches_path_with(path, &rule.pattern_options) {
                    //debug!("rule matches path {:?} -> ok={}", path, rule.ok);
                    return rule.ok;
                }
            }
        }
        true
    }
}

// an iterator to find all applicable git_ignore files
pub struct GitIgnoreFilesFinder<'a> {
    dir: &'a Path,
}
impl<'a> GitIgnoreFilesFinder<'a> {
    fn for_dir(dir: &'a Path) -> GitIgnoreFilesFinder<'a> {
        GitIgnoreFilesFinder { dir }
    }
}
impl<'a> Iterator for GitIgnoreFilesFinder<'a> {
    type Item = PathBuf; // I don't really see a way to deal with only &'a Path as join makes a PathBuf
    fn next(&mut self) -> Option<PathBuf> {
        loop {
            let ignore_file = self.dir.join(".gitignore");
            match self.dir.parent() {
                Some(parent) => {
                    self.dir = parent;
                    if ignore_file.exists() {
                        return Some(ignore_file);
                    }
                }
                None => {
                    return None;
                }
            }
        }
    }
}
