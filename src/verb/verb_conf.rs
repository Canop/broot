use {
    super::*,
    crate::{
        app::SelectionType,
        conf::toml::*,
        keys,
        command::Sequence,
        errors::ConfError,
    },
    std::convert::TryFrom,
    toml::Value,
};

/// read a TOML parsed value into a verb,
/// checking a few basic things in the process
impl TryFrom<&Value> for Verb {
    type Error = ConfError;
    fn try_from(verb_value: &Value) -> Result<Self, Self::Error> {
        let invocation = string_field(verb_value, "invocation");
        let key = string_field(verb_value, "key")
            .map(|s| keys::parse_key(&s))
            .transpose()?;
        let execution = string_field(verb_value, "execution");
        let internal = string_field(verb_value, "internal");
        let external = string_field(verb_value, "external");
        let cmd = string_field(verb_value, "cmd");
        let cmd_separator = string_field(verb_value, "cmd_separator");
        let from_shell = bool_field(verb_value, "from_shell");
        let leave_broot = bool_field(verb_value, "leave_broot");
        if leave_broot == Some(false) && from_shell == Some(true) {
            return Err(ConfError::InvalidVerbConf {
                details: "You can't simultaneously have leave_broot=false and from_shell=true".to_string(),
            });
        }
        let make_external_execution = |s| ExternalExecution::new(
            s,
            ExternalExecutionMode::from_conf(from_shell, leave_broot),
        ).with_set_working_dir(bool_field(verb_value, "set_working_dir"));
        let execution = match (execution, internal, external, cmd) {
            // old definition with "execution": we guess whether it's an internal or
            // an external
            (Some(s), None, None, None) => if s.starts_with(':') || s.starts_with(' ') {
                VerbExecution::Internal(InternalExecution::try_from(&s[1..])?)
            } else {
                VerbExecution::External(make_external_execution(s))
            }
            // "internal": the leading `:` or ` ` is optional
            (None, Some(s), None, None) => VerbExecution::Internal(
                if s.starts_with(':') || s.starts_with(' ') {
                    InternalExecution::try_from(&s[1..])?
                } else {
                    InternalExecution::try_from(&s)?
                }
            ),
            // "external": it can be about any form
            (None, None, Some(s), None) => VerbExecution::External(make_external_execution(s)),
            // "cmd": it's a sequence
            (None, None, None, Some(s)) => VerbExecution::Sequence(
                SequenceExecution {
                    sequence: Sequence::new(s, cmd_separator),
                }
            ),
            _ => {
                return Err(ConfError::InvalidVerbConf {
                    details: "You must define either internal, external or cmd".to_string(),
                });
            }
        };
        let description = string_field(verb_value, "description")
            .map(VerbDescription::from_text)
            .unwrap_or_else(|| VerbDescription::from_code(execution.to_string()));
        let mut verb = Verb::new(
            invocation.as_deref(),
            execution,
            description,
        )?;
        if let Some(key) = key {
            if keys::is_reserved(key) {
                return Err(ConfError::ReservedKey { key: keys::key_event_desc(key) });
            }
            verb = verb.with_key(key);
        }
        if let Some(shortcut) = string_field(verb_value, "shortcut") {
            verb.names.push(shortcut);
        }
        verb.selection_condition = match string_field(verb_value, "apply_to").as_deref() {
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
