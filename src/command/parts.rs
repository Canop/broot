use {
    super::PatternParts,
    crate::verb::VerbInvocation,
    std::fmt,
};

/// An intermediate parsed representation of the raw string
#[derive(Debug, Clone, PartialEq)]
pub struct CommandParts {
    pub pattern: Option<PatternParts>, //
    pub verb_invocation: Option<VerbInvocation>, // may be empty if user typed the separator but no char after
}

impl fmt::Display for CommandParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pattern) = &self.pattern {
            pattern.fmt(f)?;
        }
        if let Some(invocation) = &self.verb_invocation {
            write!(f, "{}", invocation)?;
        }
        Ok(())
    }
}

impl CommandParts {

    pub fn new() -> CommandParts {
        CommandParts {
            pattern: None,
            verb_invocation: None,
        }
    }

    pub fn from(raw: &str) -> Self {
        let mut pattern_parts: Vec<String> = vec![String::new()];
        let mut verb_invocation: Option<String> = None;
        let mut escaping = false;
        for c in raw.chars() {
            if c == '\\' {
                if escaping {
                    escaping = false;
                } else {
                    escaping = true;
                    continue;
                }
            }
            if let Some(ref mut verb_invocation) = verb_invocation {
                verb_invocation.push(c);
            } else {
                if !escaping {
                    if c == ' ' || c == ':' {
                        verb_invocation = Some(String::new());
                        continue;
                    }
                    if c == '/' {
                        pattern_parts.push(String::new());
                        continue;
                    }
                }
                let idx = pattern_parts.len()-1;
                pattern_parts[idx].push(c);
            }
            escaping = false;
        }
        let parts_len = pattern_parts.len();
        let mut drain = pattern_parts.drain(..);
        let mode = if parts_len > 1 {
            drain.next()
        } else {
            None
        };
        let pattern = drain.next().unwrap();
        let flags = drain.next();
        let pattern = if pattern.is_empty() && mode.is_none() {
            None
        } else {
            Some(PatternParts {
                mode,
                pattern,
                flags,
            })
        };
        let verb_invocation = verb_invocation.map(|s| VerbInvocation::from(&*s));
        CommandParts {
            pattern,
            verb_invocation,
        }
    }

    /// split an input into its two possible parts, the pattern
    /// and the verb invocation. Each part, when defined, is
    /// suitable to create a command on its own.
    pub fn split(mut self) -> (Option<CommandParts>, Option<CommandParts>) {
        (
            self.pattern.take().map(|p| CommandParts {
                pattern: Some(p),
                verb_invocation: None,
            }),
            self.verb_invocation.take().map(|inv| CommandParts {
                pattern: None,
                verb_invocation: Some(inv),
            }),
        )
    }

}

impl Default for CommandParts {
    fn default() -> CommandParts {
        CommandParts::new()
    }
}

#[cfg(test)]
mod command_parsing_tests {
    use super::*;
    fn check(
        raw: &str,
        mode: Option<&str>,
        pattern: Option<&str>,
        flags: Option<&str>,
        verb_invocation: Option<&str>,
    ) {
        println!("checking {:?}", raw);
        let left = CommandParts::from(raw);
        let right = CommandParts {
            pattern: pattern.map(|pattern| PatternParts {
                mode: mode.map(|s| s.to_string()),
                pattern: pattern.to_string(),
                flags: flags.map(|s| s.to_string()),
            }),
            verb_invocation: verb_invocation.map(|s| VerbInvocation::from(s)),
        };
        assert_eq!(left, right);
    }
    #[test]
    fn test_command_parsing() {
        check("", None, None, None, None);
        check(" ", None, None, None, Some(""));
        check(":", None, None, None, Some(""));
        check("pat", None, Some("pat"), None, None);
        check("pat ", None, Some("pat"), None, Some(""));
        check(" verb arg1 arg2", None, None, None, Some("verb arg1 arg2"));
        check(" verb ", None, None, None, Some("verb "));
        check("/", Some(""), Some(""), None, None);
        check("/ cp", Some(""), Some(""), None, Some("cp"));
        check("pat verb ", None, Some("pat"), None, Some("verb "));
        check("/pat/i verb ", Some(""), Some("pat"), Some("i"), Some("verb "));
        check("r/pat/i verb ", Some("r"), Some("pat"), Some("i"), Some("verb "));
        check("/pat/:verb ", Some(""), Some("pat"), Some(""), Some("verb "));
        check("/pat", Some(""), Some("pat"), None, None);
        check("p/pat", Some("p"), Some("pat"), None, None);
        check("mode/", Some("mode"), Some(""), None, None);
        check(r"c/two\ words verb /arg/u/ment", Some("c"), Some("two words"), None, Some("verb /arg/u/ment"));
        check(r"a\/slash", None, Some("a/slash"), None, None);
        check(r"p/i\/cd", Some("p"), Some("i/cd"), None, None);
        check(r"c/\\b", Some("c"), Some(r"\b"), None, None);
        check(r"c/\:\:H:cd", Some("c"), Some(r"::H"), None, Some("cd"));
    }
}
