use {
    super::*,
    crate::{
        app::SelectionType,
        errors::ConfError,
    },
    crossterm::event::KeyEvent,
    std::convert::TryFrom,
};

/// what's needed to handle a verb
#[derive(Debug)]
pub struct VerbConf {
    pub shortcut: Option<String>,
    pub invocation: Option<String>,
    pub key: Option<KeyEvent>,
    pub execution: String,
    pub description: Option<String>,
    pub from_shell: Option<bool>,
    pub leave_broot: Option<bool>,
    pub selection_condition: SelectionType,
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
            let internal_execution = InternalExecution::try_from(s)?;
            let name = verb_conf.invocation.as_ref().map(|inv| {
                let inv: &str = &inv;
                VerbInvocation::from(inv).name
            });
            Verb::new(
                name,
                VerbExecution::Internal(internal_execution),
                VerbDescription::from_code(verb_conf.execution.to_string()),
            )
        } else {
            Verb::external(
                if let Some(inv) = &verb_conf.invocation {
                    inv
                } else {
                    // can we really accept externals without invocation ? Is this supported ?
                    ""
                },
                &verb_conf.execution,
                ExternalExecutionMode::from_conf(verb_conf.from_shell, verb_conf.leave_broot),
            )?
        };
        if let Some(key) = verb_conf.key {
            verb = verb.with_key(key);
        }
        if let Some(shortcut) = &verb_conf.shortcut {
            verb.names.push(shortcut.to_string());
        }
        if let Some(description) = &verb_conf.description {
            verb.description = VerbDescription::from_text(description.to_string());
        }
        verb.selection_condition = verb_conf.selection_condition;
        Ok(verb)
    }
}
