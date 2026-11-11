//! this mod achieves the transformation of a string containing
//! one or several commands into a vec of parsed commands

use {
    super::{
        Command,
        CommandParts,
    },
    crate::{
        app::AppContext,
        program_error::ProgramError,
        verb::*,
    },
    std::fmt,
};

/// A sequence of unparsed commands
#[derive(Debug, Clone)]
pub struct Sequence {
    pub commands: Vec<String>,
    /// the separator, used to display the sequence
    pub separator: String,
}

impl Sequence {
    /// return the separator to use to parse sequences.
    pub fn local_separator() -> String {
        match std::env::var("BROOT_CMD_SEPARATOR") {
            Ok(sep) if !sep.is_empty() => sep,
            _ => String::from(";"),
        }
    }
    /// Split the raw string with the separator, or with the local
    /// separator if none is given (an empty separator doesn't split)
    pub fn new<S: Into<String>>(
        raw: S,
        separator: Option<S>,
    ) -> Self {
        let raw = raw.into();
        let separator = separator.map_or_else(Sequence::local_separator, Into::into);
        let commands = if separator.is_empty() {
            vec![raw]
        } else {
            raw.split(&separator).map(String::from).collect()
        };
        Self { commands, separator }
    }
    pub fn new_single<S: Into<String>>(cmd: S) -> Self {
        Self {
            commands: vec![cmd.into()],
            separator: String::new(),
        }
    }
    pub fn new_local(raw: String) -> Self {
        Self::new(raw, None)
    }
    /// Parse the sequence into a vec of commands.
    ///
    /// Beware: `panel_state_type` filtering isn't applied (
    ///   and would be difficult a priori as we can change panel
    ///   in the middle of a sequence)
    pub fn parse(
        &self,
        con: &AppContext,
    ) -> Result<Vec<(String, Command)>, ProgramError> {
        let mut commands = Vec::new();
        for input in &self.commands {
            add_commands(input, &mut commands, con)?;
        }
        Ok(commands)
    }
}

impl fmt::Display for Sequence {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "{}", self.commands.join(&self.separator))
    }
}

/// Add commands to a sequence.
///
/// An input may be made of two parts:
///  - a search pattern
///  - a verb followed by its arguments
///
/// We need to build a command for each part so
///   that the search is effectively done before
///   the verb invocation
fn add_commands(
    input: &str,
    commands: &mut Vec<(String, Command)>,
    con: &AppContext,
) -> Result<(), ProgramError> {
    let raw_parts = CommandParts::from(input.to_string());
    let (pattern, verb_invocation) = raw_parts.split();
    if let Some(pattern) = pattern {
        commands.push((input.to_string(), Command::from_parts(pattern, false)));
    }
    if let Some(verb_invocation) = verb_invocation {
        let mut command = Command::from_parts(verb_invocation, true);
        if let Command::VerbInvocate(invocation) = &command {
            // we check that the verb exists to avoid running a sequence
            // of actions with some missing
            match con.verb_store.search_prefix(&invocation.name, None) {
                PrefixSearchResult::NoMatch => {
                    return Err(ProgramError::UnknownVerb {
                        name: invocation.name.clone(),
                    });
                }
                PrefixSearchResult::Matches(_) => {
                    return Err(ProgramError::AmbiguousVerbName {
                        name: invocation.name.clone(),
                    });
                }
                PrefixSearchResult::Match(_, verb) => {
                    if let Some(internal) = verb.get_internal() {
                        command = Command::Internal {
                            internal,
                            input_invocation: Some(invocation.clone()),
                        };
                    }
                    commands.push((input.to_string(), command));
                }
            }
        }
    }
    Ok(())
}
