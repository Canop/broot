use {
    crate::{
        app::{
            PanelStateType,
            SelectionType,
        },
        command::Sequence,
        errors::ConfError,
        keys,
        verb::*,
    },
    serde::Deserialize,
};

/// A deserializable verb entry in the configuration
#[derive(Default, Debug, Clone, Deserialize)]
pub struct VerbConf {

    invocation: Option<String>,

    internal: Option<String>,

    external: Option<ExecPattern>,

    execution: Option<ExecPattern>,

    cmd: Option<String>,

    cmd_separator: Option<String>,

    key: Option<String>,

    #[serde(default)]
    keys: Vec<String>,

    #[serde(default)]
    extensions: Vec<String>,

    shortcut: Option<String>,

    leave_broot: Option<bool>,

    from_shell: Option<bool>,

    apply_to: Option<String>,

    set_working_dir: Option<bool>,

    working_dir: Option<String>,

    description: Option<String>,

    auto_exec: Option<bool>,

    switch_terminal: Option<bool>,

    #[serde(default)]
    panels: Vec<PanelStateType>,
}

/// read a deserialized verb conf item into a verb,
/// checking a few basic things in the process
impl VerbConf {
    /// the verb_store is provided to allow a verb to be built from other ones
    /// already defined
    pub fn make_verb(&self, previous_verbs: &[Verb]) -> Result<Verb, ConfError> {
        let vc = self;
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
        let make_external_execution = |s| {
            let working_dir = match (vc.set_working_dir, &vc.working_dir) {
                (Some(false), _) => None,
                (_, Some(s)) => Some(s.clone()),
                (Some(true), None) => Some("{directory}".to_owned()),
                (None, None) => None,
            };
            let mut external_execution = ExternalExecution::new(
                s,
                ExternalExecutionMode::from_conf(vc.from_shell, vc.leave_broot),
            )
            .with_working_dir(working_dir);
            if let Some(b) = self.switch_terminal {
                external_execution.switch_terminal = b;
            }
            external_execution
        };
        let execution = match (execution, internal, external, cmd) {
            // old definition with "execution": we guess whether it's an internal or
            // an external
            (Some(ep), None, None, None) => {
                if let Some(internal_pattern) = ep.as_internal_pattern() {
                    if let Some(previous_verb) = previous_verbs.iter().find(|&v| v.has_name(internal_pattern)) {
                        previous_verb.execution.clone()
                    } else {
                        VerbExecution::Internal(InternalExecution::try_from(internal_pattern)?)
                    }
                } else {
                    VerbExecution::External(make_external_execution(ep.clone()))
                }
            }
            // "internal": the leading `:` or ` ` is optional
            (None, Some(s), None, None) => {
                VerbExecution::Internal(if s.starts_with(':') || s.starts_with(' ') {
                    InternalExecution::try_from(&s[1..])?
                } else {
                    InternalExecution::try_from(s)?
                })
            }
            // "external": it can be about any form
            (None, None, Some(ep), None) => {
                VerbExecution::External(make_external_execution(ep.clone()))
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
        // we accept both key and keys. We merge both here
        let mut unchecked_keys = vc.keys.clone();
        if let Some(key) = &vc.key {
            unchecked_keys.push(key.clone());
        }
        let mut checked_keys = Vec::new();
        for key in &unchecked_keys {
            let key = crokey::parse(key)?;
            if keys::is_reserved(key) {
                return Err(ConfError::ReservedKey {
                    key: keys::KEY_FORMAT.to_string(key)
                });
            }
            checked_keys.push(key);
        }
        for extension in &self.extensions {
            verb.file_extensions.push(extension.clone());
        }
        if !checked_keys.is_empty() {
            verb.add_keys(checked_keys);
        }
        if let Some(shortcut) = &vc.shortcut {
            verb.names.push(shortcut.clone());
        }
        if vc.auto_exec == Some(false) {
            verb.auto_exec = false;
        }
        if !vc.panels.is_empty() {
            verb.panels = vc.panels.clone();
        }
        verb.selection_condition = match vc.apply_to.as_deref() {
            Some("file") => SelectionType::File,
            Some("directory") => SelectionType::Directory,
            Some("any") => SelectionType::Any,
            None => SelectionType::Any,
            Some(s) => {
                return Err(ConfError::InvalidVerbConf {
                    details: format!("{s:?} isn't a valid value of apply_to"),
                });
            }
        };
        Ok(verb)
    }
}

