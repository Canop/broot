use {
    crate::{
        app::SelectionType,
        keys,
        command::Sequence,
        errors::ConfError,
        verb::*,
    },
    serde::Deserialize,
    std::convert::TryFrom,
};

/// a deserializable verb entry in the configuration
#[derive(Default, Debug, Clone, Deserialize)]
pub struct VerbConf {

    invocation: Option<String>,

    internal: Option<String>,

    external: Option<String>,

    execution: Option<String>,

    cmd: Option<String>,

    cmd_separator: Option<String>,

    key: Option<String>,

    shortcut: Option<String>,

    leave_broot: Option<bool>,

    from_shell: Option<bool>,

    apply_to: Option<String>,

    set_working_dir: Option<bool>,

    description: Option<String>,

}

/// read a deserialized verb conf item into a verb,
/// checking a few basic things in the process
impl TryFrom<&VerbConf> for Verb {
    type Error = ConfError;
    fn try_from(vc: &VerbConf) -> Result<Self, Self::Error> {
        if vc.leave_broot == Some(false) && vc.from_shell == Some(true) {
            return Err(ConfError::InvalidVerbConf {
                details: "You can't simultaneously have leave_broot=false and from_shell=true".to_string(),
            });
        }
        let invocation = vc.invocation.clone().filter(|i| !i.is_empty());
        let internal = vc.internal.as_ref().filter(|i| !i.is_empty());
        let external = vc.external.as_ref().filter(|i| !i.is_empty());
        let cmd = vc.cmd.as_ref().filter(|i| !i.is_empty());
        let cmd_separator = vc.cmd_separator.as_ref().filter(|i| !i.is_empty());
        let execution = vc.execution.as_ref().filter(|i| !i.is_empty());
        let key = vc.key.clone().map(|s| keys::parse_key(&s)).transpose()?;
        let make_external_execution = |s| {
            ExternalExecution::new(
                s,
                ExternalExecutionMode::from_conf(vc.from_shell, vc.leave_broot),
            )
            .with_set_working_dir(vc.set_working_dir)
        };
        let execution = match (execution, internal, external, cmd) {
            // old definition with "execution": we guess whether it's an internal or
            // an external
            (Some(s), None, None, None) => {
                if s.starts_with(':') || s.starts_with(' ') {
                    VerbExecution::Internal(InternalExecution::try_from(&s[1..])?)
                } else {
                    VerbExecution::External(make_external_execution(s.to_string()))
                }
            }
            // "internal": the leading `:` or ` ` is optional
            (None, Some(s), None, None) => {
                VerbExecution::Internal(if s.starts_with(':') || s.starts_with(' ') {
                    InternalExecution::try_from(&s[1..])?
                } else {
                    InternalExecution::try_from(&s)?
                })
            }
            // "external": it can be about any form
            (None, None, Some(s), None) => {
                VerbExecution::External(make_external_execution(s.to_string()))
            }
            // "cmd": it's a sequence
            (None, None, None, Some(s)) => VerbExecution::Sequence(SequenceExecution {
                sequence: Sequence::new(s, cmd_separator),
            }),
            _ => {
                return Err(ConfError::InvalidVerbConf {
                    details: "You must define either internal, external or cmd".to_string(),
                });
            }
        };
        let description = vc
            .description
            .clone()
            .map(VerbDescription::from_text)
            .unwrap_or_else(|| VerbDescription::from_code(execution.to_string()));
        let mut verb = Verb::new(
            invocation.as_deref(),
            execution,
            description,
        )?;
        if let Some(key) = key {
            if keys::is_reserved(key) {
                return Err(ConfError::ReservedKey {
                    key: keys::key_event_desc(key),
                });
            }
            verb = verb.with_key(key);
        }
        if let Some(shortcut) = &vc.shortcut {
            verb.names.push(shortcut.clone());
        }
        verb.selection_condition = match vc.apply_to.as_deref() {
            Some("file") => SelectionType::File,
            Some("directory") => SelectionType::Directory,
            Some("any") => SelectionType::Any,
            None => SelectionType::Any,
            Some(s) => {
                return Err(ConfError::InvalidVerbConf {
                    details: format!("{:?} isn't a valid value of apply_to", s),
                });
            }
        };
        Ok(verb)
    }
}

