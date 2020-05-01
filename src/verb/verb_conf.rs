use {
    super::{ExternalExecutionMode, Internal, Verb},
    crate::errors::ConfError,
    crossterm::event::KeyEvent,
    std::convert::TryFrom,
};

/// what's needed to handle a verb
#[derive(Debug)]
pub struct VerbConf {
    pub shortcut: Option<String>,
    pub invocation: String,
    pub key: Option<KeyEvent>,
    pub execution: String,
    pub description: Option<String>,
    pub from_shell: Option<bool>,
    pub leave_broot: Option<bool>,
}

impl TryFrom<&VerbConf> for Verb {
    type Error = ConfError;
    fn try_from(verb_conf: &VerbConf) -> Result<Self, Self::Error> {
        // if there's a ':' or ' ' at starts, it's an internal.
        // In other cases it's an external.
        // (we might support adding aliases to externals in the
        // future. In such cases we'll check among previously
        // added externals if no internal is found with the name)
        let mut s: &str = &verb_conf.execution;
        let mut verb = if s.starts_with(':') || s.starts_with(' ') {
            s = &s[1..];
            let mut bang = false;
            if s.starts_with('!') {
                bang = true;
                s = &s[1..];
            }
            let internal = Internal::try_from(s)?; // check among known internals
            Verb::internal_bang(internal, bang)
        } else {
            Verb::external(
                &verb_conf.invocation,
                &verb_conf.execution,
                ExternalExecutionMode::from_conf(verb_conf.from_shell, verb_conf.leave_broot),
            )?
        };
        if let Some(key) = verb_conf.key {
            verb = verb.with_key(key);
        }
        verb.shortcut = verb_conf.shortcut.clone();
        verb.description = verb_conf.description.clone();
        Ok(verb)
    }
}
