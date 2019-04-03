//! this mod achieves the transformation of a string containing
//! one or several commands into a vec of parsed commands, using
//! the verbstore to try guess what part is an argument and what
//! part is a filter

use crate::app_context::AppContext;
use crate::commands::Command;
use crate::errors::ProgramError;
use crate::verb_store::PrefixSearchResult;

#[derive(Debug)]
enum CommandSequenceToken {
    Standard(String), // one or several words, not starting with a ':'. May be a filter or a verb argument
    VerbKey(String), // a verb (the ':' isn't given)
}
struct CommandSequenceTokenizer {
    chars: Vec<char>,
    pos: usize,
}
impl CommandSequenceTokenizer {
    pub fn from(sequence: &str) -> CommandSequenceTokenizer {
        CommandSequenceTokenizer {
            chars: sequence.chars().collect(),
            pos: 0
        }
    }
}
impl Iterator for CommandSequenceTokenizer {
    type Item = CommandSequenceToken;
    fn next(&mut self) -> Option<CommandSequenceToken> {
        if self.pos >= self.chars.len() {
            return None;
        }
        let is_verb = if self.chars[self.pos] == ':' {
            self.pos = self.pos + 1;
            true
        } else {
            false
        };
        let mut end = self.pos;
        let mut between_quotes = false;
        while end < self.chars.len() {
            if self.chars[end] == '"' {
                between_quotes = !between_quotes;
            } else if self.chars[end] == ' ' && !between_quotes {
                break;
            }
            end += 1;
        }
        let token: String = self.chars[self.pos..end].iter().collect();
        self.pos = end + 1;
        Some(
            if is_verb {
                CommandSequenceToken::VerbKey(token)
            } else {
                CommandSequenceToken::Standard(token)
            }
        )
    }
}

/// parse a string which is meant as a sequence of commands.
/// Note that this is inherently flawed as packing several commands
/// into a string without hard separator is ambiguous in the general
/// case.
///
/// In the future I might introduce a way to define a variable hard separator
/// (for example "::sep=#:some_filter#:some command with three arguments#a_filter")
///
/// The current parsing try to be the least possible flawed by
/// giving verbs the biggest sequence of tokens accepted by their
/// execution pattern.
pub fn parse_command_sequence(sequence: &str, con: &AppContext) -> Result<Vec<Command>, ProgramError> {
    let mut tokenizer = CommandSequenceTokenizer::from(sequence);
    let mut commands: Vec<Command> = Vec::new();
    let mut leftover: Option<CommandSequenceToken> = None;
    loop {
        let first_token = if let Some(token) = leftover.take().or_else(|| tokenizer.next()) {
            token
        } else {
            break;
        };
        let raw = match first_token {
            CommandSequenceToken::VerbKey(key) => {
                let verb = match con.verb_store.search(&key) {
                    PrefixSearchResult::NoMatch => {
                        return Err(ProgramError::UnknownVerb{key});
                    }
                    PrefixSearchResult::TooManyMatches => {
                        return Err(ProgramError::AmbiguousVerbKey{key});
                    }
                    PrefixSearchResult::Match(verb) => verb
                };
                let mut raw = format!(":{}", key);
                if let Some(args_regex) = &verb.args_parser {
                    let mut args: Vec<String> = Vec::new();
                    let mut nb_valid_args = 0;
                    // we'll try to consume as many tokens as possible
                    while let Some(token) = tokenizer.next() {
                        match token {
                            CommandSequenceToken::VerbKey(_) => {
                                leftover = Some(token);
                                break;
                            }
                            CommandSequenceToken::Standard(raw) => {
                                args.push(raw);
                                if args_regex.is_match(&args.join(" ")) {
                                    nb_valid_args = args.len();
                                }
                            }
                        }
                    }
                    if nb_valid_args == 0 && !args_regex.is_match("") {
                        return Err(ProgramError::UnmatchingVerbArgs{key});
                    }
                    for (i, arg) in args.drain(..).enumerate() {
                        if i < nb_valid_args {
                            raw.push(' ');
                            raw.push_str(&arg);
                        } else {
                            commands.push(Command::from(arg));
                        }
                    }
                }
                raw
            }
            CommandSequenceToken::Standard(raw) => raw
        };
        commands.push(Command::from(raw));
    }
    if let Some(token) = leftover.take() {
        commands.push(Command::from(match token{
            CommandSequenceToken::Standard(raw) => raw,
            CommandSequenceToken::VerbKey(raw) => format!(":{}", raw),
        }));
    }
    Ok(commands)
}

