//! this mod achieves the transformation of a string containing
//! one or several commands into a vec of parsed commands

use {
    super::{Command, CommandParts},
    crate::{
        app::AppContext,
        errors::ProgramError,
        verb::PrefixSearchResult,
    },
};

/// parse a string which is meant as a sequence of commands.
///
/// The ';' separator is used to identify inputs unless it's
/// overriden in env variable BROOT_CMD_SEPARATOR.
/// Verbs are verified, to ensure the command sequence has
/// no unexpected holes.
pub fn parse_command_sequence<'a>(
    sequence: &'a str,
    con: &AppContext,
) -> Result<Vec<(&'a str, Command)>, ProgramError> {
    let separator = match std::env::var("BROOT_CMD_SEPARATOR") {
        Ok(sep) if !sep.is_empty() => sep,
        _ => String::from(";"),
    };
    debug!("Splitting cmd sequence with {:?}", separator);
    let mut commands = Vec::new();
    for input in sequence.split(&separator) {
        // an input may be made of two parts:
        //  - a search pattern
        //  - a verb followed by its arguments
        // we need to build a command for each part so
        // that the search is effectively done before
        // the verb invocation
        let raw_parts = CommandParts::from(input);
        let (pattern, verb_invocation) = raw_parts.split();
        if let Some(pattern) = pattern {
            debug!("adding pattern: {:?}", pattern);
            commands.push((input, Command::from_parts(&pattern, false)));
        }
        if let Some(verb_invocation) = verb_invocation {
            debug!("adding verb_invocation: {:?}", verb_invocation);
            let command = Command::from_parts(&verb_invocation, true);
            if let Command::VerbInvocate(invocation) = &command {
                // we check that the verb exists to avoid running a sequence
                // of actions with some missing
                match con.verb_store.search(&invocation.name) {
                    PrefixSearchResult::NoMatch => {
                        return Err(ProgramError::UnknownVerb {
                            name: invocation.name.to_string(),
                        });
                    }
                    PrefixSearchResult::Matches(_) => {
                        return Err(ProgramError::AmbiguousVerbName {
                            name: invocation.name.to_string(),
                        });
                    }
                    _ => {}
                }
                commands.push((input, command));
            }
        }
    }
    Ok(commands)
}

