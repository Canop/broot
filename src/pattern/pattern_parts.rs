use {
    std::fmt,
};

/// An intermediate parsed representation of the raw string making
/// a pattern, with up to 3 parts (search mode, core pattern, modifiers)
#[derive(Debug, Clone, PartialEq)]
pub struct PatternParts {
    /// can't be empty by construct
    parts: Vec<String>,
}

impl fmt::Display for PatternParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.parts.len() {
            1 => write!(f, "{}", &self.parts[0]),
            2 => write!(f, "{}/{}", &self.parts[0], &self.parts[1]),
            _ => write!(f, "{}/{}/{}", &self.parts[0], &self.parts[1], &self.parts[2]),
        }
    }
}

impl Default for PatternParts {
    fn default() -> Self {
        Self {
            parts: vec![String::new()],
        }
    }
}

#[cfg(test)]
impl TryFrom<&[&str]> for PatternParts {
    type Error = &'static str;
    fn try_from(a: &[&str]) -> Result<Self, Self::Error> {
        if a.is_empty() {
            return Err("invalid empty parts array");
        }
        let parts = a.iter().map(|s| (*s).into()).collect();
        Ok(Self { parts })
    }
}

impl PatternParts {
    pub fn push(&mut self, c: char) {
        // self.parts can't be empty, by construct
        self.parts.last_mut().unwrap().push(c);
    }
    pub fn is_between_slashes(&self) -> bool {
        self.parts.len() == 2
    }
    pub fn add_part(&mut self) {
        self.parts.push(String::new());
    }
    pub fn is_empty(&self) -> bool {
        self.core().is_empty()
    }
    pub fn core(&self) -> &str {
        if self.parts.len() > 1 {
            &self.parts[1]
        } else {
            &self.parts[0]
        }
    }
    pub fn mode(&self) -> Option<&String> {
        if self.parts.len() > 1 {
            self.parts.get(0)
        } else {
            None
        }
    }
    pub fn flags(&self) -> Option<&str> {
        if self.parts.len() > 2 {
            self.parts.get(2).map(|s| s.as_str())
        } else {
            None
        }
    }
}

