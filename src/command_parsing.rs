//! this mod achieves the transformation of a string containing
//! one or several commands into a vec of parsed commands, using
//! the verbstore to try guess what part is an argument and what
//! part is a filter
//!
//! It currently uses the following syntax:
//!
//! A command string is a series of commands. The first command can optionally
//! be a search command. A verb command follows a search with a space or
//! semicolon, just like the UI. Subsequent verb commands may follow, separated
//! by a separator, which defaults to semicolon
//!
//! Examples:
//!
//! search1 :verb1 arg; verb2;:verb3
use {
    crate::{
        commands::Command,
        errors::ProgramError,
        verb_store::{PrefixSearchResult, VerbStore},
    },
    nom::{
        bytes::complete::{is_not, tag},
        character::complete::{char, space0},
        combinator::{opt, rest, verify},
        sequence::{delimited, preceded, separated_pair, terminated},
        IResult,
    },
};

// Parse a separator, surrounded by 0 or more whitespace
fn parse_separator<'a>(separator: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str, ()> {
    delimited(space0, tag(separator), space0)
}

// Parse the initial search pattern
fn parse_search_pattern(input: &str) -> IResult<&str, &str, ()> {
    verify(terminated(is_not(" \t:"), space0), |pat: &str| {
        !pat.is_empty()
    })(input)
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ParsedCommandParts<'a> {
    /// The verb part, without the prefix colon or whitespace.
    verb: &'a str,

    /// The args part, with whitespace trimmed.
    args: &'a str,

    /// The entire command: the verb and the args, including the colon if present.
    command: &'a str,
}

/// Parse a verb and its arguments, up to the next separator. Returns a nom
/// parsing function that returns:
/// - The verb
/// - the arguments
/// - the entire command
///
/// For instance, given:
///
/// :command a b c; :cmd2 a b c
///
/// Assuming ; is the separator, it will return:
///
/// ("command", "a b c", :command a b c")
///
/// The verb part is used to look up the verb in the verb.
/// The args part is used to validate the arguments
/// The command part is used to construct a command. Be sure to prepend a :
/// if there isn't one already there.
/// Fails if the input is empty.
fn parse_command_parts<'a>(
    separator: &'a str,
) -> impl Fn(&'a str) -> IResult<&'a str, ParsedCommandParts<'a>, ()> {
    move |input: &'a str| {
        let (command, suffix) = match input.find(separator) {
            None => (input, &input[input.len()..]),
            Some(index) => (&input[..index], &input[index..]),
        };

        let command = command.trim();

        let parse_command_parts = separated_pair(
            // The verb part
            preceded(opt(char(':')), is_not(" \t")),
            // Some whitespace
            space0,
            // The args
            rest,
        );

        parse_command_parts(command).map(move |(_, (verb, args))| {
            (
                suffix,
                ParsedCommandParts {
                    verb,
                    args,
                    command,
                },
            )
        })
    }
}

/// parse a string which is meant as a sequence of commands.
///
/// Syntax:
///
/// A single search string, followed by any number of commands. The search
/// string is separated from the first command by a whitespace or semicolon,
/// just like in the interactive interface; subsequent commands are separated
/// by the `separator`, which the CLI defaults to ";"
pub fn parse_command_sequence(
    separator: &str,
    input: &str,
    verb_store: &VerbStore,
) -> Result<Vec<Command>, ProgramError> {
    let mut commands = Vec::new();

    let parse_command = parse_command_parts(separator);
    let parse_sep = parse_separator(separator);

    // First, parse the initial search pattern
    let mut input = match opt(parse_search_pattern)(input) {
        Ok((tail, Some(pattern))) => {
            commands.push(Command::from(pattern.to_string()));
            tail
        }
        Ok((tail, None)) => tail,

        // parse_search_pattern contains an opt, which means it either
        // succeeds with Some or succeeds with None. It shouldn't be able to
        // fail.
        Err(..) => panic!("Error parsing the --cmd search pattern; this shouldn't be possible"),
    };

    while !input.is_empty() {
        input = match parse_command(input) {
            Ok((tail, parsed_verb)) => {
                // Lookup the verb
                let verb = parsed_verb.verb;
                let verb_spec = match verb_store.search(verb) {
                    PrefixSearchResult::NoMatch => {
                        return Err(ProgramError::UnknownVerb {
                            name: verb.to_string(),
                        })
                    }
                    PrefixSearchResult::TooManyMatches(..) => {
                        return Err(ProgramError::AmbiguousVerbName {
                            name: verb.to_string(),
                        })
                    }
                    PrefixSearchResult::Match(spec) => spec,
                };

                // Validate the arguments. If a parser is present, use it;
                // otherwise, this verb wants no arguments.
                if !match verb_spec.args_parser.as_ref() {
                    Some(args_regex) => args_regex.is_match(parsed_verb.args),
                    None => parsed_verb.args.is_empty(),
                } {
                    return Err(ProgramError::UnmatchingVerbArgs {
                        name: verb.to_string(),
                    });
                }

                // Construct the command
                let command = parsed_verb.command;
                let command = if command.starts_with(':') {
                    command.to_string()
                } else {
                    format!(":{}", command)
                };
                let command = Command::from(command);
                commands.push(command);

                // Consume a separator, then continue the loop
                match parse_sep(tail) {
                    Ok((tail, _)) => tail,
                    Err(_) => {
                        // If this parser errored, no separator was found, which
                        // should only be possible if the input is empty.
                        debug_assert!(tail.is_empty());
                        break;
                    }
                }
            }

            // To the best of my knowledge, the parser can't fail on
            // non-empty input, so we panic
            Err(..) => panic!("Unexpected parsing error while parsing commands"),
        }
    }

    Ok(commands)
}

#[cfg(test)]
mod command_parsing_tests {
    // TODO: add tests. Need to mock up VerbStore for this.
}
