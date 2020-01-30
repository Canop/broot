//! Implements parsing and applying .gitignore files.

use {
    git2,
    glob,
    id_arena::{
        Arena,
        Id,
    },
    regex::Regex,
    std::{
        fs::File,
        io::{BufRead, BufReader, Result},
        path::Path,
    },
};

pub fn is_repo(root: &Path) -> bool {
    root.join(".git").exists()
}

/// a simple rule of a gitignore file
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
            rules.len(),
        );
        Ok(GitIgnoreFile { rules })
    }
}

pub fn find_global_ignore() -> Option<GitIgnoreFile> {
    let global_conf = match git2::Config::open_default() {
        Ok(conf) => conf,
        Err(e) => {
            debug!("open default git config failed : {:?}", e);
            return None;
        }
    };
    global_conf.get_path("core.excludesfile")
        .ok()
        .as_ref()
        .and_then(
            |path| time!(
                Debug,
                "GitIgnoreFile::new",
                GitIgnoreFile::new(path)
            ).ok()
        )
}

#[derive(Debug, Clone, Default)]
pub struct GitIgnoreChain {
    file_ids: Vec<Id<GitIgnoreFile>>,
}
impl GitIgnoreChain {
    pub fn push(&mut self, id: Id<GitIgnoreFile>) {
        self.file_ids.push(id);
    }
}
pub struct GitIgnorer {
    files: Arena<GitIgnoreFile>,
    global_chain: GitIgnoreChain,
}
impl GitIgnorer {
    pub fn new() -> Self {
        let mut files = Arena::new();
        let mut global_chain = GitIgnoreChain::default();
        if let Some(gif) = find_global_ignore() {
            global_chain.push(files.alloc(gif));
        }
        Self {
            files,
            global_chain,
        }
    }
    pub fn root_chain(&mut self, mut dir: &Path) -> GitIgnoreChain {
        debug!("searching applicable gifs for {:?}", dir);
        let mut chain = self.global_chain.clone();
        loop {
            debug!("  looking in {:?}", dir);
            let ignore_file = dir.join(".gitignore");
            if let Ok(gif) = GitIgnoreFile::new(&ignore_file) {
                debug!("  adding {:?}", &ignore_file);
                chain.push(self.files.alloc(gif));
            }
            if is_repo(dir) {
                debug!("  break because git repo");
                break;
            }
            if let Some(parent) = dir.parent() {
                dir = parent;
            } else {
                break;
            }
        }
        chain
    }
    pub fn deeper_chain(
        &mut self,
        parent_chain: &GitIgnoreChain,
        dir: &Path,
    ) -> GitIgnoreChain {
        // if the current folder is a repository, then
        // we reset the chain to the root one:
        // we don't want the .gitignore files of super repositories
        // (see https://github.com/Canop/broot/issues/160)
        let mut chain = if is_repo(dir) {
            //debug!("entering a git repo {:?}", dir);
            self.global_chain.clone()
        } else {
            //debug!("subfolder {:?} in same repo", dir);
            parent_chain.clone()
        };
        let ignore_file = dir.join(".gitignore");
        if let Ok(gif) = GitIgnoreFile::new(&ignore_file) {
            chain.push(self.files.alloc(gif));
        }
        chain
    }
    pub fn accepts(
        &self,
        chain: &GitIgnoreChain,
        path: &Path,
        filename: &str,
        directory: bool,
    ) -> bool {
        // we start with deeper files: deeper rules have a bigger priority
        for id in chain.file_ids.iter().rev() {
            let file = &self.files[*id];
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

