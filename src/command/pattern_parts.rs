use {
    std::fmt,
};

/// An intermediate parsed representation of the raw string
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatternParts {
    pub mode: Option<String>, // may be Some("") if the user typed `/pat`
    pub pattern: String, // either a fuzzy pattern or the core of a regex
    pub flags: Option<String>, // may be Some("") if user asked for a regex but specified no flag
}

impl fmt::Display for PatternParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(mode) = &self.mode {
            write!(f, "{}/", mode)?;
        }
        write!(f, "{}", self.pattern)?;
        if let Some(flags) = &self.flags {
            write!(f, "/{}", flags)?;
        }
        Ok(())
    }
}
