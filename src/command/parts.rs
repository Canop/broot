use {
    crate::verb::VerbInvocation,
    std::fmt,
};

/// An intermediate parsed representation of the raw string
#[derive(Debug, Clone)]
pub struct CommandParts {
    pub pattern: Option<String>, // either a fuzzy pattern or the core of a regex
    pub regex_flags: Option<String>, // may be Some("") if user asked for a regex but specified no flag
    pub verb_invocation: Option<VerbInvocation>, // may be empty if user typed the separator but no char after
}

impl fmt::Display for CommandParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(pattern) = &self.pattern {
            write!(f, "{}", pattern)?;
            if let Some(flags) = &self.regex_flags {
                write!(f, "/{}", flags)?;
            }
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
            regex_flags: None,
            verb_invocation: None,
        }
    }

    pub fn from(raw: &str) -> Self {
        let mut cp = CommandParts::new();
        let c = regex!(
            r"(?x)
                ^
                (?P<slash_before>/)?
                (?P<pattern>[^\s/:]+)?
                (?:/(?P<regex_flags>\w*))?
                (?:[\s:]+(?P<verb_invocation>.*))?
                $
            "
        )
        .captures(raw);
        if let Some(c) = c {
            if let Some(pattern) = c.name("pattern") {
                cp.pattern = Some(String::from(pattern.as_str()));
                if let Some(rxf) = c.name("regex_flags") {
                    cp.regex_flags = Some(String::from(rxf.as_str()));
                } else if c.name("slash_before").is_some() {
                    cp.regex_flags = Some("".into());
                }
            }
            if let Some(verb) = c.name("verb_invocation") {
                cp.verb_invocation = Some(VerbInvocation::from(verb.as_str()));
            }
        } else {
            // Non matching pattterns include "///"
            // We decide the whole is a fuzzy search pattern, in this case
            // (this will change when we release the new input syntax)
            cp.pattern = Some(String::from(raw));
        }
        cp
    }

    /// split an input into its two possible parts, the pattern
    /// and the verb invocation. Each part, when defined, is
    /// suitable to create a command on its own.
    pub fn split(raw: &str) -> (Option<String>, Option<String>) {
        let captures = regex!(
            r"(?x)
                ^
                (?P<pattern_part>/?[^\s/:]+/?\w*)?
                (?P<verb_part>[\s:]+(.+))?
                $
            "
        )
        .captures(raw)
        .unwrap(); // all parts optional : always captures
        (
            captures
                .name("pattern_part")
                .map(|c| c.as_str().to_string()),
            captures.name("verb_part").map(|c| c.as_str().to_string()),
        )
    }
}

impl Default for CommandParts {
    fn default() -> CommandParts {
        CommandParts::new()
    }
}
