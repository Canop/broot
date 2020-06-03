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
        let mut cp = CommandParts::new();
        let c = regex!(
            r"(?x)
                ^
                (?:(?P<search_mode>\w*)/)?
                (?P<pattern>[^\s/:]*)?
                (?:/(?P<pattern_flags>\w*))?
                (?:[\s:]+(?P<verb_invocation>.*))?
                $
            "
        )
        .captures(raw);
        if let Some(c) = c {
            if let Some(pattern) = c.name("pattern") {
                let pattern = pattern.as_str().to_string();
                let mode_match = c.name("search_mode");
                let has_pattern = !pattern.is_empty();
                let has_mode = mode_match.map_or(false, |c| !c.as_str().is_empty());
                if  has_pattern || has_mode {
                    cp.pattern = Some(PatternParts {
                        mode: c.name("search_mode").map(|c| c.as_str().to_string()),
                        pattern: pattern.as_str().to_string(),
                        flags: c.name("pattern_flags").map(|c| c.as_str().to_string()),
                    });
                }
            }
            if let Some(verb) = c.name("verb_invocation") {
                cp.verb_invocation = Some(VerbInvocation::from(verb.as_str()));
            }
        } else {
            // Non matching pattterns include "///"
            // We decide the whole is a search pattern, in this case
            cp.pattern = Some(PatternParts::default())
        }
        cp
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
        check("pat", None, Some("pat"), None, None);
        check("pat ", None, Some("pat"), None, Some(""));
        check(" verb arg1 arg2", None, None, None, Some("verb arg1 arg2"));
        check(" verb ", None, None, None, Some("verb "));
        check("pat verb ", None, Some("pat"), None, Some("verb "));
        check("/pat/i verb ", Some(""), Some("pat"), Some("i"), Some("verb "));
        check("r/pat/i verb ", Some("r"), Some("pat"), Some("i"), Some("verb "));
        check("/pat/:verb ", Some(""), Some("pat"), Some(""), Some("verb "));
        check("/pat", Some(""), Some("pat"), None, None);
        check("p/pat", Some("p"), Some("pat"), None, None);
        check("mode/", Some("mode"), Some(""), None, None);
    }
}
