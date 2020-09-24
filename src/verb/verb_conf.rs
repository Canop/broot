use {
    super::*,
    crate::{
        app::SelectionType,
        command::Sequence,
        errors::ConfError,
    },
    crossterm::event::KeyEvent,
    std::convert::TryFrom,
};

#[derive(Debug)]
pub enum VerbExecutionType {
    Internal,
    External,
    Sequence,
}


/// what's needed to handle a verb
///
/// A verb must contain either a `cmd` (a sequence)
/// or an `execution` (call to an internal or external
/// definition)
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

    pub execution_type: VerbExecutionType,

    /// the separator to use when splitting the sequence
    /// (only makes sense when the execution is a sequence)
    pub cmd_separator: Option<String>,
}

impl TryFrom<&VerbConf> for Verb {
    type Error = ConfError;
    fn try_from(verb_conf: &VerbConf) -> Result<Self, Self::Error> {
        let execution = match verb_conf.execution_type {
            VerbExecutionType::Internal => VerbExecution::Internal(
                InternalExecution::try_from(&verb_conf.execution[1..])?
            ),
            VerbExecutionType::External => VerbExecution::External(
                ExternalExecution::new(
                    &verb_conf.execution,
                    ExternalExecutionMode::from_conf(
                        verb_conf.from_shell,
                        verb_conf.leave_broot,
                    ),
                )
            ),
            VerbExecutionType::Sequence => VerbExecution::Sequence(
                SequenceExecution {
                    sequence: Sequence::new(
                        verb_conf.execution.to_string(),
                        verb_conf.cmd_separator.clone(),
                    )
                }
            ),
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
