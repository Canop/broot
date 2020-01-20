use crossterm::event::KeyEvent;

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
    pub confirm: Option<bool>,
}

