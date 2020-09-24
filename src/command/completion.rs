
use {
    super::CommandParts,
    crate::{
        app::{
            AppContext,
            Selection,
        },
        path,
        path_anchor::PathAnchor,
        verb::PrefixSearchResult,
    },
    std::io,
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

    fn from_list(
        completions: Vec<String>,
    ) -> Self {
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
                if w.starts_with(start) {
                    &w[start.len()..]
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
        sel: Selection<'_>,
    ) -> Self {
        match con.verb_store.search(start, Some(sel.stype)) {
            PrefixSearchResult::NoMatch => {
                Self::None
            }
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

    fn for_path(
        anchor: PathAnchor,
        arg: &str,
        _con: &AppContext,
        sel: Selection<'_>,
    ) -> io::Result<Self> {
        let c = regex!(r"^(.*?)([^/]*)$").captures(arg).unwrap();
        let parent_part = &c[1];
        let child_part = &c[2];
        let parent = path::path_from(sel.path, anchor, parent_part);
        if !parent.exists() {
            debug!("no path completion possible because {:?} doesn't exist", &parent);
            return Ok(Self::None);
        }
        let mut children = Vec::new();
        for entry in parent.read_dir()? {
            let entry = entry?;
            let mut name = entry.file_name().to_string_lossy().to_string();
            if !child_part.is_empty() {
                if !name.starts_with(child_part) {
                    continue;
                }
                if name==child_part && entry.file_type()?.is_dir() {
                    name = "/".to_string();
                } else {
                    name.drain(0..child_part.len());
                }
            }
            children.push(name);
        }
        Ok(Self::from_list(children))
    }

    fn for_arg(
        verb_name: &str,
        arg: &str,
        con: &AppContext,
        sel: Selection<'_>,
    ) -> Self {
        // in the future we might offer completion of other types
        // of arguments, maybe user supplied, but there's no use case
        // now so we'll just assume the user wants to complete a path.
        if arg.contains(' ') {
            Self::None
        } else {
            let anchor = match con.verb_store.search(verb_name, Some(sel.stype)) {
                PrefixSearchResult::Match(_, verb) => verb.get_arg_anchor(),
                _ => PathAnchor::Unspecified,
            };
            match Self::for_path(anchor, arg, con, sel) {
                Ok(c) => c,
                Err(e) => {
                    warn!("Error while trying to complete path: {:?}", e);
                    Self::None
                }
            }
        }
    }

    pub fn for_input(
        parts: &CommandParts,
        con: &AppContext,
        sel: Selection<'_>,
    ) -> Self {
        match &parts.verb_invocation {
            Some(invocation) if !invocation.is_empty() => {
                match &invocation.args {
                    None => {
                        // looking into verb completion
                        Self::for_verb(&invocation.name, con, sel)
                    }
                    Some(args) if !args.is_empty() => {
                        // looking into arg completion
                        Self::for_arg(&invocation.name, args, con, sel)
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
