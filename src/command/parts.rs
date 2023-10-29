use {
    crate::{
        pattern::*,
        verb::VerbInvocation,
    },
    bet::BeTree,
    std::fmt,
};

/// An intermediate parsed representation of the raw string
#[derive(Debug, Clone, PartialEq)]
pub struct CommandParts {
    pub raw_pattern: String, // may be empty
    pub pattern: BeTree<PatternOperator, PatternParts>,
    pub verb_invocation: Option<VerbInvocation>, // may be empty if user typed the separator but no char after
}

impl fmt::Display for CommandParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.raw_pattern)?;
        if let Some(invocation) = &self.verb_invocation {
            write!(f, "{invocation}")?;
        }
        Ok(())
    }
}

impl CommandParts {
    pub fn has_not_empty_verb_invocation(&self) -> bool {
        self.verb_invocation
            .as_ref()
            .map_or(false, |vi| !vi.is_empty())
    }
    pub fn from<S: Into<String>>(raw: S) -> Self {
        let mut raw = raw.into();
        let mut invocation_start_pos: Option<usize> = None;
        let mut pt = BeTree::new();
        let mut chars = raw.char_indices().peekable();
        let mut escape_cur_char = false;
        let mut escape_next_char = false;
        // we loop on chars and build the pattern tree until we reach an unescaped ' ' or ':'
        while let Some((pos, cur_char)) = chars.next() {
            let between_slashes = pt.current_atom()
                .map_or(
                    false,
                    |pp: &PatternParts| pp.is_between_slashes(),
                );
            match cur_char {
                c if escape_cur_char => {
                    // Escaping is used to prevent characters from being consumed at the
                    // composite pattern level (or, and, parens) or as the separator between
                    // the pattern and the verb. An escaped char is usable in a pattern atom.
                    pt.mutate_or_create_atom(PatternParts::default).push(c);
                }
                '\\' => {
                    // Pattern escaping rules:
                    // 	- when after '/': only ' ', ':',  '/' and '\' need escaping
                    // 	- otherwise, '&,' '|', '(', ')' need escaping too ('(' is only here for
                    // 	symmetry)
                    let between_slashes = match pt.current_atom() {
                        Some(pattern_parts) => pattern_parts.is_between_slashes(),
                        None => false,
                    };
                    escape_next_char = match chars.peek() {
                        None => false, // End of the string, we can't be escaping
                        Some((_, next_char)) => match (next_char, between_slashes) {
                            (' ' | ':' | '/' | '\\', _) => true,
                            ('&' | '|' | '!' | '(' | ')', false) => true,
                            _ => false,
                        }
                    };
                    if !escape_next_char {
                        // if the '\' isn't used for escaping, it's used as its char value
                        pt.mutate_or_create_atom(PatternParts::default).push('\\');
                    }
                }
                ':' => {
                    if matches!(chars.peek(), Some((_,':'))) {
                        // two successive ':' in pattern position are part of the
                        // pattern
                        pt.mutate_or_create_atom(PatternParts::default).push(':');
                        escape_next_char = true;
                    } else {
                        // ending the pattern part
                        invocation_start_pos = Some(pos);
                        break;
                    }
                }
                ' ' => { // ending the pattern part
                    invocation_start_pos = Some(pos);
                    break;
                }
                '/' => { // starting an atom part
                    pt.mutate_or_create_atom(PatternParts::default).add_part();
                }
                '|' if !between_slashes && pt.accept_binary_operator() => {
                    pt.push_operator(PatternOperator::Or);
                }
                '&' if !between_slashes && pt.accept_binary_operator() => {
                    pt.push_operator(PatternOperator::And);
                }
                '!' if !between_slashes && pt.accept_unary_operator() => {
                    pt.push_operator(PatternOperator::Not);
                }
                '(' if !between_slashes && pt.accept_opening_par() => {
                    pt.open_par();
                }
                ')' if !between_slashes && pt.accept_closing_par() => {
                    pt.close_par();
                }
                _ => {
                    pt.mutate_or_create_atom(PatternParts::default).push(cur_char);
                }
            }
            escape_cur_char = escape_next_char;
            escape_next_char = false;
        }
        let mut verb_invocation = None;
        if let Some(pos) = invocation_start_pos {
            verb_invocation = Some(VerbInvocation::from(
                raw[pos + 1..].trim_start() // allowing extra spaces
            ));
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

#[cfg(test)]
mod test_command_parts {

    use {
        crate::{
            command::CommandParts,
            pattern::*,
            verb::VerbInvocation,
        },
        bet::{BeTree, Token},
    };

    fn pp(a: &[&str]) -> PatternParts {
        a.try_into().unwrap()
    }

    /// Check that the input is parsed as expected:
    /// - a raw pattern
    /// - the token (operators and pattern_parts) of the pattern
    /// - the verb invocation
    fn check(
        input: &str,
        raw_pattern: &str,
        mut pattern_tokens: Vec<Token<PatternOperator, PatternParts>>,
        verb_invocation: Option<&str>,
    ) {
        let mut pattern = BeTree::new();
        for token in pattern_tokens.drain(..) {
            pattern.push(token);
        }
        let left = CommandParts {
            raw_pattern: raw_pattern.to_string(),
            pattern,
            verb_invocation: verb_invocation.map(VerbInvocation::from),
        };
        dbg!(&left);
        let right = CommandParts::from(input);
        dbg!(&right);
        assert_eq!(left, right);
    }

    #[test]
    fn parse_empty() {
        check(
            "",
            "",
            vec![],
            None,
        );
    }
    #[test]
    fn parse_just_semicolon() {
        check(
            ":",
            "",
            vec![],
            Some(""),
        );
    }
    #[test]
    fn parse_no_pattern() {
        check(
            " cd /",
            "",
            vec![],
            Some("cd /"),
        );
    }
    #[test]
    fn parse_pattern_and_invocation() {
        check(
            "/r cd /",
            "/r",
            vec![
                Token::Atom(pp(&["", "r"])),
            ],
            Some("cd /"),
        );
    }
    #[test]
    fn allow_extra_spaces_before_invocation() {
        check(
            "  cd /",
            "",
            vec![],
            Some("cd /"),
        );
        check(
            "/r  cd /",
            "/r",
            vec![
                Token::Atom(pp(&["", "r"])),
            ],
            Some("cd /"),
        );
        check(
            r#"a\ b   e"#,
            r#"a\ b"#,
            vec![
                Token::Atom(pp(&["a b"])),
            ],
            Some("e"),
        );
        check(
            "/a:   b",
            "/a",
            vec![
                Token::Atom(pp(&["", "a"])),
            ],
            Some("b"),
        );
    }
    #[test]
    fn parse_pattern_between_slashes() {
        check(
            r#"/&"#,
            r#"/&"#,
            vec![
                Token::Atom(pp(&["", "&"])),
            ],
            None,
        );
        check(
            r#"/&/&r/a(\w-)+/ rm"#,
            r#"/&/&r/a(\w-)+/"#,
            vec![
                Token::Atom(pp(&["", "&", ""])),
                Token::Operator(PatternOperator::And),
                Token::Atom(pp(&["r", r#"a(\w-)+"#, ""])),
            ],
            Some("rm"),
        );
    }
    #[test]
    fn parse_pattern_with_space() {
        check(
            r#"a\ b"#,
            r#"a\ b"#,
            vec![
                Token::Atom(pp(&["a b"])),
            ],
            None,
        );
    }
    #[test]
    fn parse_pattern_with_slash() {
        check(
            r#"r/a\ b\//i cd /"#,
            r#"r/a\ b\//i"#,
            vec![
                Token::Atom(pp(&["r", "a b/", "i"])),
            ],
            Some("cd /"),
        );
    }
    #[test]
    fn parse_fuzzy_pattern_searching_parenthesis() {
        check(
            r#"\("#,
            r#"\("#,
            vec![
                Token::Atom(pp(&["("])),
            ],
            None,
        );
    }
    #[test]
    fn parse_regex_pattern_searching_parenthesis() {
        check(
            r#"/\("#,
            r#"/\("#,
            vec![
                Token::Atom(pp(&["", r#"\("#])),
            ],
            None,
        );
    }
    #[test]
    fn parse_composite_pattern() {
        check(
            "(/txt$/&!truc)&c/rex",
            "(/txt$/&!truc)&c/rex",
            vec![
                Token::OpeningParenthesis,
                Token::Atom(pp(&["", "txt$", ""])),
                Token::Operator(PatternOperator::And),
                Token::Operator(PatternOperator::Not),
                Token::Atom(pp(&["truc"])),
                Token::ClosingParenthesis,
                Token::Operator(PatternOperator::And),
                Token::Atom(pp(&["c", "rex"])),
            ],
            None
        );
    }
    #[test]
    fn parse_unclosed_composite_pattern() {
        check(
            r#"!/\.json$/&(c/isize/|c/i32:rm"#,
            r#"!/\.json$/&(c/isize/|c/i32"#,
            vec![
                Token::Operator(PatternOperator::Not),
                Token::Atom(pp(&["", r#"\.json$"#, ""])),
                Token::Operator(PatternOperator::And),
                Token::OpeningParenthesis,
                Token::Atom(pp(&["c", "isize", ""])),
                Token::Operator(PatternOperator::Or),
                Token::Atom(pp(&["c", "i32"])),
            ],
            Some("rm"),
        );
    }
    #[test]
    fn issue_592() { // https://github.com/Canop/broot/issues/592
        check(
            r#"\t"#,
            r#"\t"#,
            vec![
                Token::Atom(pp(&[r#"\t"#])),
            ],
            None,
        );
        check(
            r#"r/@(\.[^.]+)+/ cp .."#,
            r#"r/@(\.[^.]+)+/"#,
            vec![
                Token::Atom(pp(&["r", r#"@(\.[^.]+)+"#, ""])),
            ],
            Some("cp .."),
        );
    }
    // two colons in pattern positions are something the user searches
    #[test]
    fn allow_non_escaped_double_colon() {
        check(
            r#"::"#,
            r#"::"#,
            vec![
                Token::Atom(pp(&[r#"::"#])),
            ],
            None,
        );
        check(
            r#":::"#,
            r#"::"#,
            vec![
                Token::Atom(pp(&[r#"::"#])),
            ],
            Some(""),
        );
        check(
            r#":::cd c:\"#,
            r#"::"#,
            vec![
                Token::Atom(pp(&[r#"::"#])),
            ],
            Some(r#"cd c:\"#),
        );
        check(
            r#"and::Sc:cd c:\"#,
            r#"and::Sc"#,
            vec![
                Token::Atom(pp(&[r#"and::Sc"#])),
            ],
            Some(r#"cd c:\"#),
        );
        check(
            r#"!:: "#,
            r#"!::"#,
            vec![
                Token::Operator(PatternOperator::Not),
                Token::Atom(pp(&[r#"::"#])),
            ],
            Some(""),
        );
        check(
            r#"::a:rm"#,
            r#"::a"#,
            vec![
                Token::Atom(pp(&[r#"::a"#])),
            ],
            Some("rm"),
        );
    }
}

