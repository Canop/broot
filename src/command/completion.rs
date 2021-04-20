use {
    super::CommandParts,
    crate::{
        app::{
            AppContext,
            SelectionType,
            SelInfo,
        },
        path::{self, PathAnchor},
        verb::PrefixSearchResult,
    },
    std::{
        io,
        path::Path,
    },
};

/// find the longest common start of a and b
fn common_start<'l>(a: &'l str, b: &str) -> &'l str {
    for i in 0..a.len().min(b.len()) {
        if a.as_bytes()[i] != b.as_bytes()[i] {
            return &a[..i];
        }
    }
    a
}

/// how an input can be completed
#[derive(Debug)]
pub enum Completions {

    /// no completion found
    None,

    /// all possible completions have this common root
    Common(String),

    /// a list of possible completions
    List(Vec<String>),
}

impl Completions {
    fn from_list(completions: Vec<String>) -> Self {
        if completions.is_empty() {
            return Self::None;
        }
        let mut iter = completions.iter();
        let mut common: &str = match iter.next() {
            Some(s) => &s,
            _ => { return Self::None; }
        };
        for c in iter {
            common = common_start(common, c);
        }
        if common.is_empty() {
            Self::List(completions)
        } else {
            Self::Common(common.to_string())
        }

    }

    /// the wholes are assumed to all start with start.
    fn for_wholes(
        start: &str,
        wholes: Vec<&str>,
    ) -> Self {
        let completions = wholes.iter()
            .map(|w|
                if let Some(stripped) = w.strip_prefix(start) {
                    stripped
                } else {
                    // this might become a feature but right now it's a bug
                    warn!("unexpected non completing whole: {:?}", w);
                    *w
                }
            )
            .map(|c| c.to_string())
            .collect();
        Self::from_list(completions)
    }

    fn for_verb(
        start: &str,
        con: &AppContext,
        sel_info: SelInfo<'_>,
    ) -> Self {
        match con.verb_store.search(start, sel_info.common_stype()) {
            PrefixSearchResult::NoMatch => Self::None,
            PrefixSearchResult::Match(name, _) => {
                if start.len() >= name.len() {
                    debug_assert!(name == start);
                    Self::None
                } else {
                    Self::Common(name[start.len()..].to_string())
                }
            }
            PrefixSearchResult::Matches(completions) => Self::for_wholes(
                start,
                completions,
            ),
        }
    }

    fn list_for_path(
        verb_name: &str,
        arg: &str,
        path: &Path,
        stype: SelectionType,
        con: &AppContext,
    ) -> io::Result<Vec<String>> {
        let anchor = match con.verb_store.search(verb_name, Some(stype)) {
            PrefixSearchResult::Match(_, verb) => verb.get_arg_anchor(),
            _ => PathAnchor::Unspecified,
        };
        let c = regex!(r"^(.*?)([^/]*)$").captures(arg).unwrap();
        let parent_part = &c[1];
        let child_part = &c[2];
        let parent = path::path_from(path, anchor, parent_part);
        let mut children = Vec::new();
        if !parent.exists() {
            debug!("no path completion possible because {:?} doesn't exist", &parent);
        } else {
            for entry in parent.read_dir()? {
                let entry = entry?;
                let mut name = entry.file_name().to_string_lossy().to_string();
                if !child_part.is_empty() {
                    if !name.starts_with(child_part) {
                        continue;
                    }
                    if name == child_part && entry.file_type()?.is_dir() {
                        name = "/".to_string();
                    } else {
                        name.drain(0..child_part.len());
                    }
                }
                children.push(name);
            }
        }
        Ok(children)
    }

    fn for_arg(
        verb_name: &str,
        arg: &str,
        con: &AppContext,
        sel_info: SelInfo<'_>,
    ) -> Self {
        // in the future we might offer completion of other types
        // of arguments, maybe user supplied, but there's no use case
        // now so we'll just assume the user wants to complete a path.
        if arg.contains(' ') {
            return Self::None;
        }
        match sel_info {
            SelInfo::None => Self::None,
            SelInfo::One(sel) => {
                match Self::list_for_path(verb_name, arg, sel.path, sel.stype, con) {
                    Ok(list) => Self::from_list(list),
                    Err(e) => {
                        warn!("Error while trying to complete path: {:?}", e);
                        Self::None
                    }
                }
            }
            SelInfo::More(stage) => {
                // We're looking for the possible completions which
                // are valid for all elements of the stage
                let mut lists = stage.paths()
                    .iter()
                    .filter_map(|path| {
                        Self::list_for_path(
                                verb_name,
                                arg,
                                path,
                                SelectionType::from(path),
                                con
                        ).ok()
                    });
                let mut list = match lists.next() {
                    Some(list) => list,
                    None => {
                        // can happen if there were IO errors on paths in stage, for example
                        // on removals
                        return Self::None;
                    }
                };
                for ol in lists.next() {
                    list = list.iter().filter(|c| ol.contains(c)).cloned().collect();
                    if list.is_empty() {
                        break;
                    }
                }
                Self::from_list(list)
            }
        }
    }

    pub fn for_input(
        parts: &CommandParts,
        con: &AppContext,
        sel_info: SelInfo<'_>,
    ) -> Self {
        match &parts.verb_invocation {
            Some(invocation) if !invocation.is_empty() => {
                match &invocation.args {
                    None => {
                        // looking into verb completion
                        Self::for_verb(&invocation.name, con, sel_info)
                    }
                    Some(args) if !args.is_empty() => {
                        // looking into arg completion
                        Self::for_arg(&invocation.name, args, con, sel_info)
                    }
                    _ => {
                        // nothing possible
                        Self::None
                    }
                }
            }
            _ => Self::None, // no possible completion if no verb invocation
        }
    }

}
