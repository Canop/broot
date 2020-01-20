//! Implements parsing and applying .gitignore files.
//! Also manages a stack of such files, because more than one
//!  can apply for a dir (i.e when entering a directory we
//!  may add a gitignore file to the stack)

use {
    glob,
    regex::Regex,
    std::{
        fs::File,
        io::{BufRead, BufReader, Result},
        path::Path,
    },
};

/// a simple rule of a gitignore file
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
        if line.starts_with('#') {
            return None; // comment line
        }
        let r = regex!(
            r"(?x)
            ^\s*
            (!)?    # 1 : negation
            (.+?)   # 2 : pattern
            (/)?    # 3 : directory
            \s*$
            "
        );
        if let Some(c) = r.captures(line) {
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

/// The rules of a gitignore file
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

/// A stack of the gitignore files applying to a directory.
#[derive(Clone)]
pub struct GitIgnoreFilter {
    pub files: Vec<GitIgnoreFile>, // the last one is the deepest one
}
impl GitIgnoreFilter {
    pub fn applicable_to(mut dir: &Path) -> GitIgnoreFilter {
        let mut filter = GitIgnoreFilter { files: Vec::new() };
        debug!("searching applicable gifs for {:?}", dir);
        loop {
            debug!("  looking in {:?}", dir);
            let ignore_file = dir.join(".gitignore");
            if let Ok(gif) = GitIgnoreFile::new(&ignore_file) {
                debug!("  adding {:?}", &ignore_file);
                filter.files.push(gif);
            }
            if is_git_repo(dir) {
                debug!("  break because git repo");
                break;
            }
            if let Some(parent) = dir.parent() {
                dir = parent;
            } else {
                break;
            }
        }
        filter
    }
    pub fn extended_to(
        &self,
        dir: &Path,
    ) -> GitIgnoreFilter {
        let ignore_file = dir.join(".gitignore");
        if let Ok(gif) = GitIgnoreFile::new(&ignore_file) {
            // if the current folder is a repository, then
            // we reset the chain: we don't want the .gitignore
            // files of super repositories
            // (see https://github.com/Canop/broot/issues/160)
            let mut files = if is_git_repo(dir) {
                // we'll assume it's a .git folder
                debug!("entering a git repo {:?}", dir);
                self.files.clone()
            } else {
                debug!("subfolder {:?} in same repo", dir);
                Vec::new()
            };
            files.push(gif);
            GitIgnoreFilter { files }
        } else {
            self.clone()
        }
    }
    pub fn accepts(&self, path: &Path, filename: &str, directory: bool) -> bool {
        for file in &self.files {
            for rule in &file.rules {
                if rule.directory && !directory {
                    continue;
                }
                let ok = if rule.filename {
                    rule.pattern.matches_with(filename, rule.pattern_options)
                } else {
                    rule.pattern.matches_path_with(path, rule.pattern_options)
                };
                if ok {
                    return rule.ok;
                }
            }
        }
        true
    }
}

fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
}
