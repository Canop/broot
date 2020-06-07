
use {
    super::*,
    crate::{
        errors::PatternError,
    },
    std::{
        fmt::{self, Write},
        mem,
    },
};

/// a pattern for filtering and sorting filenames.
/// It's backed either by a fuzzy pattern matcher or
///  by a regular expression (in which case there's no real
///  score).
/// It applies either to the name, or to the sub-path.
#[derive(Debug, Clone)]
pub enum Pattern {
    None,
    NameFuzzy(FuzzyPattern),
    PathFuzzy(FuzzyPattern),
    NameRegex(RegexPattern),
    PathRegex(RegexPattern),
    Content(ContentPattern),
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::NameFuzzy(fp) => write!(f, "NameFuzzy({})", fp),
            Pattern::PathFuzzy(fp) => write!(f, "PathFuzzy({})", fp),
            Pattern::NameRegex(rp) => write!(f, "NameRegex({})", rp),
            Pattern::PathRegex(rp) => write!(f, "PathRegex({})", rp),
            Pattern::Content(rp) => write!(f, "Content({})", rp),
            Pattern::None => write!(f, "None"),
        }
    }
}

impl Pattern {
    pub fn new(mode: SearchMode, pat: &str, flags: &Option<String>) -> Result<Self, PatternError> {
        Ok(if pat.is_empty() {
            Pattern::None
        } else {
            match mode {
                SearchMode::NameFuzzy => Self::NameFuzzy(FuzzyPattern::from(pat)),
                SearchMode::PathFuzzy => Self::PathFuzzy(FuzzyPattern::from(pat)),
                SearchMode::NameRegex => Self::NameRegex(RegexPattern::from(pat, flags.as_deref().unwrap_or(""))?),
                SearchMode::PathRegex => Self::PathRegex(RegexPattern::from(pat, flags.as_deref().unwrap_or(""))?),
                SearchMode::Content => Self::Content(ContentPattern::from(pat)),
            }
        })
    }
    pub fn mode(&self) -> Option<SearchMode> {
        match self {
            Self::NameFuzzy(_) => Some(SearchMode::NameFuzzy),
            Self::PathFuzzy(_) => Some(SearchMode::PathFuzzy),
            Self::NameRegex(_) => Some(SearchMode::NameRegex),
            Self::PathRegex(_) => Some(SearchMode::PathRegex),
            Self::Content(_) => Some(SearchMode::Content),
            _ => None,
        }
    }
    pub fn object(&self) -> PatternObject {
        match self {
            Self::PathFuzzy(_) | Self::PathRegex(_) => PatternObject::FileSubpath,
            Self::Content(_) => PatternObject::FileContent,
            _ => PatternObject::FileName,
        }
    }
    /// find the position of the match, if possible
    /// (makes sense for tree rendering)
    pub fn find(&self, candidate: &str) -> Option<Match> {
        match self {
            Self::NameFuzzy(fp) => fp.find(candidate),
            Self::PathFuzzy(fp) => fp.find(candidate),
            Self::NameRegex(rp) => rp.find(candidate),
            Self::PathRegex(rp) => rp.find(candidate),
            _ => Some(Match {
                score: 1,
                pos: Vec::with_capacity(0),
            }),
        }
    }
    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        match self {
            Pattern::NameFuzzy(fp) => fp.score_of(&candidate.name),
            Pattern::PathFuzzy(fp) => fp.score_of(&candidate.name),
            Pattern::NameRegex(rp) => rp.find(&candidate.name).map(|m| m.score),
            Pattern::PathRegex(rp) => rp.find(&candidate.name).map(|m| m.score),
            Pattern::Content(cp) => cp.score_of(candidate),
            Pattern::None => Some(1),
        }
    }
    pub fn is_some(&self) -> bool {
        match self {
            Pattern::None => false,
            _ => true,
        }
    }
    pub fn as_input(&self, search_mode_map: &SearchModeMap) -> String {
        let mut input = String::new();
        if let Some(mode_key) = self.mode().and_then(|mode| search_mode_map.key(mode)) {
            input.push_str(mode_key);
            input.push('/');
        }
        match self {
            Pattern::NameFuzzy(fp) | Pattern::PathFuzzy(fp) => {
                write!(input, "{}", &fp).unwrap();
            }
            Pattern::NameRegex(rp) | Pattern::PathRegex(rp) => {
                write!(input, "{}", &rp).unwrap();
            }
            _ => {}
        }
        input
    }
    /// empties the pattern and return it
    /// Similar to Option::take
    pub fn take(&mut self) -> Pattern {
        mem::replace(self, Pattern::None)
    }
    /// return the number of results we should find before starting to
    ///  sort them (unless time is runing out).
    pub fn optimal_result_number(&self, targeted_size: usize) -> usize {
        match self {
            Pattern::NameFuzzy(fp) => fp.optimal_result_number(targeted_size),
            Pattern::PathFuzzy(fp) => fp.optimal_result_number(targeted_size),
            Pattern::NameRegex(rp) => rp.optimal_result_number(targeted_size),
            Pattern::PathRegex(rp) => rp.optimal_result_number(targeted_size),
            Pattern::Content(cp) => cp.optimal_result_number(targeted_size),
            Pattern::None => targeted_size,
        }
    }
}

/// A Match is a positive result of pattern matching
#[derive(Debug, Clone)]
pub struct Match {
    pub score: i32, // score of the match, guaranteed strictly positive, bigger is better
    pub pos: Vec<usize>, // positions of the matching chars
}
