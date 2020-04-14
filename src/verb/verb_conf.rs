use {
    crate::{
        errors::ConfError,
    },
    crossterm::event::KeyEvent,
    std::convert::TryFrom,
    super::{
        Verb,
        VerbExecution,
    },
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
        let execution = VerbExecution::try_from(&verb_conf.execution)?;
        let mut verb = match execution {
            VerbExecution::Internal{ internal, bang } => {
                Verb::internal_bang(internal, bang)
            }
            VerbExecution::External(_) => {
                Verb::external(
                    &verb_conf.invocation,
                    &verb_conf.execution,
                )?
            }
        };
        if let Some(key) = verb_conf.key {
            verb = verb.with_key(key);
        }
        verb.shortcut = verb_conf.shortcut.clone();
        verb.description = verb_conf.description.clone();
        if let Some(b) = verb_conf.from_shell.as_ref() {
            verb.from_shell = *b;
        }
        if let Some(b) = verb_conf.leave_broot.as_ref() {
            verb.leave_broot = *b;
        }
        Ok(verb)
    }
}
