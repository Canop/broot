use {
    super::CommandParts,
    crate::{
        app::{
            AppContext,
            SelInfo,
        },
        path::{self, PathAnchor},
        syntactic::SYNTAX_THEMES,
        verb::{
            ArgDef,
            PrefixSearchResult,
        },
    },
    lazy_regex::regex_captures,
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
            Some(s) => s,
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
        match con.verb_store.search_sel_info(start, sel_info) {
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
        sel_info: SelInfo<'_>,
        con: &AppContext,
    ) -> io::Result<Vec<String>> {
        let anchor = match con.verb_store.search_sel_info(verb_name, sel_info) {
            PrefixSearchResult::Match(_, verb) => verb.get_unique_arg_anchor(),
            _ => PathAnchor::Unspecified,
        };
        let (_, parent_part, child_part) = regex_captures!(r"^(.*?)([^/]*)$", arg).unwrap();
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


    /// we have a verb, we try to complete one of the args
    fn for_arg(
        verb_name: &str,
        arg: &str,
        con: &AppContext,
        sel_info: SelInfo<'_>,
    ) -> Self {
        if arg.contains(' ') {
            return Self::None;
        }
        // we try to get the type of argument
        let arg_def = con
            .verb_store
            .search_sel_info_unique(verb_name, sel_info)
            .and_then(|verb| verb.invocation_parser.as_ref())
            .and_then(|invocation_parser| invocation_parser.get_unique_arg_def());
        if matches!(arg_def, Some(ArgDef::Theme)) {
            Self::for_theme_arg(arg)
        } else {
            Self::for_path_arg(verb_name, arg, con, sel_info)
        }
    }

    /// we have a verb and it asks for a theme
    fn for_theme_arg(
        arg: &str,
    ) -> Self {
        let arg = arg.to_lowercase();
        let completions: Vec<String> = SYNTAX_THEMES
            .iter()
            .map(|st| st.name().to_lowercase())
            .filter_map(|name| name.strip_prefix(&arg).map(|s| s.to_string()))
            .collect();
        Self::from_list(completions)
    }

    /// we have a verb and it asks for a path
    fn for_path_arg(
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
        match &sel_info {
            SelInfo::None => Self::None,
            SelInfo::One(sel) => {
                match Self::list_for_path(verb_name, arg, sel.path, sel_info, con) {
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
                                sel_info,
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
                for ol in lists {
                    list.retain(|c| ol.contains(c));
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
        info!("Looking for completions");
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
