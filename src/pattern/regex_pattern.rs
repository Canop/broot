//! a filtering pattern using a regular expression

use {
    super::NameMatch,
    crate::errors::PatternError,
    lazy_regex::regex,
    smallvec::SmallVec,
    std::fmt,
};

#[derive(Debug, Clone)]
pub struct RegexPattern {
    rex: regex::Regex,
    flags: String,
}

impl fmt::Display for RegexPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.flags.is_empty() {
            write!(f, "/{}", self.rex)
        } else {
            write!(f, "/{}/{}", self.rex, self.flags)
        }
    }
}

impl RegexPattern {
    pub fn from(pat: &str, flags: &str) -> Result<Self, PatternError> {
        Ok(RegexPattern {
            rex: super::build_regex(pat, flags)?,
            flags: flags.to_string(),
        })
    }
    /// return a match if the pattern can be found in the candidate string
    pub fn find(&self, candidate: &str) -> Option<NameMatch> {
        // note that there's no significative cost related to using
        //  find over is_match
        self.rex.find(candidate).map(|rm| {
            let mut pos = SmallVec::with_capacity(rm.end() - rm.start());
            for i in rm.start()..rm.end() {
                pos.push(i);
            }
            super::NameMatch { score: 1, pos }
        })
    }
    pub fn is_empty(&self) -> bool {
        self.rex.as_str().is_empty()
    }

}
