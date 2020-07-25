
use {
    super::*,
    crate::{
        app::AppContext,
        content_search::ContentMatch,
        errors::PatternError,
    },
    bet::BeTree,
    std::{
        path::Path,
    },
};

/// a pattern for filtering and sorting files.
#[derive(Debug, Clone)]
pub enum Pattern {
    None,
    NameFuzzy(FuzzyPattern),
    PathFuzzy(FuzzyPattern),
    NameRegex(RegexPattern),
    PathRegex(RegexPattern),
    Content(ContentPattern),
    Composite(CompositePattern),
}

impl Pattern {

    pub fn new(
        raw_expr: &BeTree<PatternOperator, PatternParts>,
        con: &AppContext,
    ) -> Result<Self, PatternError> {
        let expr: BeTree<PatternOperator, Pattern> = raw_expr
            .try_map_atoms::<_, PatternError, _>(|pattern_parts| {
                let core = pattern_parts.core();
                Ok(
                    if core.is_empty() {
                        Pattern::None
                    } else {
                        let parts_mode = pattern_parts.mode();
                        let mode = con.search_modes.search_mode(parts_mode)?;
                        let flags = pattern_parts.flags();
                        match mode {
                            SearchMode::NameFuzzy => Self::NameFuzzy(FuzzyPattern::from(core)),
                            SearchMode::PathFuzzy => Self::PathFuzzy(FuzzyPattern::from(core)),
                            SearchMode::NameRegex => {
                                Self::NameRegex(RegexPattern::from(core, flags.as_deref().unwrap_or(""))?)
                            }
                            SearchMode::PathRegex => {
                                Self::PathRegex(RegexPattern::from(core, flags.unwrap_or(""))?)
                            }
                            SearchMode::Content => Self::Content(ContentPattern::from(core)),
                        }
                    }
                )
            })?;
        Ok(if expr.is_empty() {
            Pattern::None
        } else if expr.is_atomic() {
            expr.atoms().pop().unwrap()
        } else {
            Self::Composite(CompositePattern::new(expr))
        })
    }

    pub fn object(&self) -> PatternObject {
        let mut object = PatternObject::default();
        match self {
            Self::None => {}
            Self::PathFuzzy(_) | Self::PathRegex(_) => {
                object.subpath = true;
            }
            Self::NameFuzzy(_) | Self::NameRegex(_) => {
                object.name = true;
            }
            Self::Content(_) => {
                object.content = true;
            }
            Self::Composite(cp) => {
                for atom in cp.expr.iter_atoms() {
                    object |= atom.object();
                }
            }
        }
        object
    }

    pub fn search_string(
        &self,
        candidate: &str,
    ) -> Option<NameMatch> {
        match self {
            Self::NameFuzzy(fp) => fp.find(candidate),
            Self::NameRegex(rp) => rp.find(candidate),
            Self::PathFuzzy(fp) => fp.find(candidate),
            Self::PathRegex(rp) => rp.find(candidate),
            Self::Composite(cp) => cp.search_string(candidate),
            _ => None,
        }
    }

    pub fn search_content(
        &self,
        candidate: &Path,
        desired_len: usize, // available space for content match display
    ) -> Option<ContentMatch> {
        match self {
            Self::Content(cp) => cp.get_content_match(candidate, desired_len),
            Self::Composite(cp) => cp.search_content(candidate, desired_len),
            _ => None,
        }
    }

    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        match self {
            Pattern::NameFuzzy(fp) => fp.score_of(&candidate.name),
            Pattern::PathFuzzy(fp) => fp.score_of(&candidate.subpath),
            Pattern::NameRegex(rp) => rp.find(&candidate.name).map(|m| m.score),
            Pattern::PathRegex(rp) => rp.find(&candidate.subpath).map(|m| m.score),
            Pattern::Content(cp) => cp.score_of(candidate),
            Pattern::Composite(cp) => cp.score_of(candidate),
            Pattern::None => Some(1),
        }
    }

    pub fn score_of_string(&self, candidate: &str) -> Option<i32> {
        match self {
            Pattern::NameFuzzy(fp) => fp.score_of(&candidate),
            Pattern::PathFuzzy(fp) => fp.score_of(&candidate),
            Pattern::NameRegex(rp) => rp.find(&candidate).map(|m| m.score),
            Pattern::PathRegex(rp) => rp.find(&candidate).map(|m| m.score),
            Pattern::Content(_) => None, // this isn't suitable
            Pattern::Composite(cp) => cp.score_of_string(candidate),
            Pattern::None => Some(1),
        }
    }

    pub fn is_some(&self) -> bool {
        match self {
            Pattern::None => false,
            _ => true,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Pattern::None => true,
            _ => false,
        }
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
            _ => targeted_size,
        }
    }

    ///
    pub fn get_content_pattern(&self) -> Option<&ContentPattern> {
        match self {
            Pattern::Content(cp) => Some(cp),
            Pattern::Composite(cp) => cp.get_content_pattern(),
            _ => None,
        }
    }
}

