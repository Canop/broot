use {
    crate::{
        app::PanelStateType,
        verb::*,
    },
    serde::{
        Deserialize,
        Serialize,
    },
};

/// A deserializable verb entry in the configuration
#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct VerbConf {
    pub invocation: Option<String>,

    pub internal: Option<String>,

    pub external: Option<ExecPattern>,

    pub execution: Option<ExecPattern>,

    pub cmd: Option<String>,

    pub cmd_separator: Option<String>,

    pub key: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keys: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extensions: Vec<String>,

    pub shortcut: Option<String>,

    pub leave_broot: Option<bool>,

    pub from_shell: Option<bool>,

    #[serde(default, skip_serializing_if = "FileTypeCondition::is_default")]
    pub apply_to: FileTypeCondition,

    pub set_working_dir: Option<bool>,

    pub working_dir: Option<String>,

    pub description: Option<String>,

    pub auto_exec: Option<bool>,

    pub switch_terminal: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panels: Vec<PanelStateType>,

    pub refresh_after: Option<bool>,
}
