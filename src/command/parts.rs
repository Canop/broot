use {
    crate::{
        pattern::*,
        verb::VerbInvocation,
    },
    bet::BeTree,
    std::fmt,
};

/// An intermediate parsed representation of the raw string
#[derive(Debug, Clone)]
pub struct CommandParts {
    pub raw_pattern: String, // may be empty
    pub pattern: BeTree<PatternOperator, PatternParts>, //
    pub verb_invocation: Option<VerbInvocation>, // may be empty if user typed the separator but no char after
}

impl fmt::Display for CommandParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_pattern)?;
        if let Some(invocation) = &self.verb_invocation {
            write!(f, "{}", invocation)?;
        }
        Ok(())
    }
}

impl CommandParts {

    pub fn from(
        mut raw: String,
    ) -> Self {
        //let mut verb_invocation: Option<String> = None;
        let mut invocation_start_pos: Option<usize> = None;
        let mut escaping = false;
        let mut pt = BeTree::new();
        for (pos, c) in raw.char_indices() {
            if c == '\\' {
                if escaping {
                    escaping = false;
                } else {
                    escaping = true;
                    continue;
                }
            }
            if !escaping {
                if c == ' ' || c == ':' {
                    invocation_start_pos = Some(pos);
                    break;
                }
                if c == '/' {
                    pt.mutate_or_create_atom(PatternParts::new).add_part();
                    continue;
                }
                let allow_inter_pattern_token = match pt.current_atom() {
                    Some(pattern_parts) => pattern_parts.allow_inter_pattern_token(),
                    None => true,
                };
                if allow_inter_pattern_token {
                    match c {
                        '|' => {
                            pt.push_operator(PatternOperator::Or);
                            continue;
                        }
                        '&' => {
                            pt.push_operator(PatternOperator::And);
                            continue;
                        }
                        '!' => {
                            pt.push_operator(PatternOperator::Not);
                            continue;
                        }
                        '(' => {
                            pt.open_par();
                            continue;
                        }
                        ')' => {
                            pt.close_par();
                            continue;
                        }
                        _ => {}
                    }
                }
            }
            pt.mutate_or_create_atom(PatternParts::new).push(c);
            escaping = false;
        }
        let mut verb_invocation = None;
        if let Some(pos) = invocation_start_pos {
            verb_invocation = Some(VerbInvocation::from(&raw[pos+1..]));
            raw.truncate(pos);
        }
        CommandParts {
            raw_pattern: raw,
            pattern: pt,
            verb_invocation,
        }
    }

    /// split an input into its two possible parts, the pattern
    /// and the verb invocation. Each part, when defined, is
    /// suitable to create a command on its own.
    pub fn split(mut self) -> (Option<CommandParts>, Option<CommandParts>) {
        let verb_invocation = self.verb_invocation.take();
        (
            if self.raw_pattern.is_empty() {
                None
            } else {
                Some(CommandParts {
                    raw_pattern: self.raw_pattern,
                    pattern: self.pattern,
                    verb_invocation: None,
                })
            },
            verb_invocation.map(|verb_invocation| CommandParts {
                raw_pattern: String::new(),
                pattern: BeTree::new(),
                verb_invocation: Some(verb_invocation),
            }),
        )
    }

}

