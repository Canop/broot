//! this mod achieves the transformation of a string containing
//! one or several commands into a vec of parsed commands

use {
    crate::{
        app_context::AppContext,
        commands::{
            Action,
            Command, CommandParts,
        },
        errors::ProgramError,
        verb_store::PrefixSearchResult,
    },
};

/// parse a string which is meant as a sequence of commands.
///
/// The ';' separator is used to identify inputs unless it's
/// overriden in env variable BROOT_CMD_SEPARATOR.
/// Verbs are verified, to ensure the command sequence has
/// no unexpected holes.
pub fn parse_command_sequence(
    sequence: &str,
    con: &AppContext,
) -> Result<Vec<Command>, ProgramError> {
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
        let (pattern, verb_invocation) = CommandParts::split(input);
        if let Some(pattern) = pattern {
            debug!("adding pattern: {:?}", pattern);
            commands.push(Command::from_raw(pattern, false));
        }
        if let Some(verb_invocation) = verb_invocation {
            debug!("adding verb_invocation: {:?}", verb_invocation);
            let command = Command::from_raw(verb_invocation, true);
            if let Action::VerbInvocate(invocation) = &command.action {
                // we check that the verb exists to avoid running a sequence
                // of actions with some missing
                match con.verb_store.search(&invocation.name) {
                    PrefixSearchResult::NoMatch => {
                        return Err(ProgramError::UnknownVerb {
                            name: invocation.name.to_string(),
                        });
                    }
                    PrefixSearchResult::TooManyMatches(_) => {
                        return Err(ProgramError::AmbiguousVerbName {
                            name: invocation.name.to_string(),
                        });
                    }
                    _ => {}
                }
                commands.push(command);
            }
        }
    }
    Ok(commands)
}
