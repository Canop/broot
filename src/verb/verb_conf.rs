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
    pub set_working_dir: Option<bool>,
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
        let execution = if s.starts_with(':') || s.starts_with(' ') {
            s = &s[1..];
            VerbExecution::Internal(InternalExecution::try_from(s)?)
        } else {
            VerbExecution::External(ExternalExecution::new(
                &verb_conf.execution,
                ExternalExecutionMode::from_conf(
                    verb_conf.from_shell,
                    verb_conf.leave_broot,
                ),
            ))
        };
        let description = if let Some(description) = &verb_conf.description {
            VerbDescription::from_text(description.to_string())
        } else {
            VerbDescription::from_code(verb_conf.execution.to_string())
        };
        let mut verb = Verb::new(
            verb_conf.invocation.as_deref(),
            execution,
            description,
        )?;
        if let Some(key) = verb_conf.key {
            verb = verb.with_key(key);
        }
        if let Some(shortcut) = &verb_conf.shortcut {
            verb.names.push(shortcut.to_string());
        }
        if let Some(b) = verb_conf.set_working_dir {
            verb.set_working_dir(b);
        }
        verb.selection_condition = verb_conf.selection_condition;
        Ok(verb)
    }
}
