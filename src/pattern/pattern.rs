use {
    super::*,
    crate::{
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
    NameExact(ExactPattern),
    NameFuzzy(FuzzyPattern),
    NameRegex(RegexPattern),
    NameTokens(TokPattern),
    PathExact(ExactPattern),
    PathFuzzy(FuzzyPattern),
    PathRegex(RegexPattern),
    PathTokens(TokPattern),
    ContentExact(ContentExactPattern),
    ContentRegex(ContentRegexPattern),
    Composite(CompositePattern),
}

impl Pattern {

    pub fn new(
        raw_expr: &BeTree<PatternOperator, PatternParts>,
        search_modes: &SearchModeMap,
        content_search_max_file_size: usize,
    ) -> Result<Self, PatternError> {
        let expr: BeTree<PatternOperator, Pattern> = raw_expr
            .try_map_atoms::<_, PatternError, _>(|pattern_parts| {
                let core = pattern_parts.core();
                Ok(
                    if core.is_empty() {
                        Pattern::None
                    } else {
                        let parts_mode = pattern_parts.mode();
                        let mode = search_modes.search_mode(parts_mode)?;
                        let flags = pattern_parts.flags();
                        match mode {
                            SearchMode::NameExact => Self::NameExact(
                                ExactPattern::from(core)
                            ),
                            SearchMode::NameFuzzy => Self::NameFuzzy(
                                FuzzyPattern::from(core)
                            ),
                            SearchMode::NameRegex => Self::NameRegex(
                                RegexPattern::from(core, flags.unwrap_or(""))?
                            ),
                            SearchMode::NameTokens => Self::NameTokens(
                                TokPattern::new(core)
                            ),
                            SearchMode::PathExact => Self::PathExact(
                                ExactPattern::from(core)
                            ),
                            SearchMode::PathFuzzy => Self::PathFuzzy(
                                FuzzyPattern::from(core)
                            ),
                            SearchMode::PathRegex => Self::PathRegex(
                                RegexPattern::from(core, flags.unwrap_or(""))?
                            ),
                            SearchMode::PathTokens => Self::PathTokens(
                                TokPattern::new(core)
                            ),
                            SearchMode::ContentExact => Self::ContentExact(
                                ContentExactPattern::new(core, content_search_max_file_size)
                            ),
                            SearchMode::ContentRegex => Self::ContentRegex(
                                ContentRegexPattern::new(
                                    core,
                                    flags.unwrap_or(""),
                                    content_search_max_file_size,
                                )?
                            ),
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
            Self::NameExact(_) | Self::NameFuzzy(_) | Self::NameRegex(_) | Self::NameTokens(_) => {
                object.name = true;
            }
            Self::PathExact(_) | Self::PathFuzzy(_) | Self::PathRegex(_) | Self::PathTokens(_) => {
                object.subpath = true;
            }
            Self::ContentExact(_) | Self::ContentRegex(_) => {
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
            Self::NameExact(ep) | Self::PathExact(ep) => ep.find(candidate),
            Self::NameFuzzy(fp) | Self::PathFuzzy(fp) => fp.find(candidate),
            Self::NameRegex(rp) | Self::PathRegex(rp) => rp.find(candidate),
            Self::NameTokens(tp) | Self::PathTokens(tp) => tp.find(candidate),
            Self::Composite(cp) => cp.search_string(candidate),
            _ => None,
        }
    }

    /// find the content to show next to the name of the file
    /// when the search involved a content filtering
    pub fn search_content(
        &self,
        candidate: &Path,
        desired_len: usize, // available space for content match display
    ) -> Option<ContentMatch> {
        match self {
            Self::ContentExact(cp) => cp.get_content_match(candidate, desired_len),
            Self::ContentRegex(cp) => cp.get_content_match(candidate, desired_len),
            Self::Composite(cp) => cp.search_content(candidate, desired_len),
            _ => None,
        }
    }

    /// get the line of the first match, if any
    pub fn get_match_line_count(
        &self,
        path: &Path,
    ) -> Option<usize> {
        match self {
            Self::ContentExact(cp) => cp.get_match_line_count(path),
            Self::ContentRegex(cp) => cp.get_match_line_count(path),
            Self::Composite(cp) => cp.get_match_line_count(path),
            _ => None,
        }
    }

    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        match self {
            Self::NameExact(ep) => ep.score_of(candidate.name),
            Self::NameFuzzy(fp) => fp.score_of(candidate.name),
            Self::NameRegex(rp) => rp.find(candidate.name).map(|m| m.score),
            Self::NameTokens(tp) => tp.score_of(candidate.name),
            Self::PathExact(ep) => ep.score_of(candidate.subpath),
            Self::PathFuzzy(fp) => fp.score_of(candidate.subpath),
            Self::PathRegex(rp) => rp.find(candidate.subpath).map(|m| m.score),
            Self::PathTokens(tp) => tp.score_of(candidate.subpath),
            Self::ContentExact(cp) => cp.score_of(candidate),
            Self::ContentRegex(cp) => cp.score_of(candidate),
            Self::Composite(cp) => cp.score_of(candidate),
            Self::None => Some(1),
        }
    }

    pub fn score_of_string(&self, candidate: &str) -> Option<i32> {
        match self {
            Self::NameExact(ep) => ep.score_of(candidate),
            Self::NameFuzzy(fp) => fp.score_of(candidate),
            Self::NameRegex(rp) => rp.find(candidate).map(|m| m.score),
            Self::NameTokens(tp) => tp.score_of(candidate),
            Self::PathExact(ep) => ep.score_of(candidate),
            Self::PathFuzzy(fp) => fp.score_of(candidate),
            Self::PathRegex(rp) => rp.find(candidate).map(|m| m.score),
            Self::PathTokens(tp) => tp.score_of(candidate),
            Self::ContentExact(_) => None, // this isn't suitable
            Self::ContentRegex(_) => None, // this isn't suitable
            Self::Composite(cp) => cp.score_of_string(candidate),
            Self::None => Some(1),
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_empty()
    }

    /// an empty pattern is one which doesn't discriminate
    /// (it accepts everything)
    pub fn is_empty(&self) -> bool {
        match self {
            Self::NameExact(ep) | Self::PathExact(ep) => ep.is_empty(),
            Self::ContentExact(ep) => ep.is_empty(),
            Self::NameFuzzy(fp) | Self::PathFuzzy(fp) => fp.is_empty(),
            Self::NameRegex(rp) | Self::PathRegex(rp) => rp.is_empty(),
            Self::ContentRegex(rp) => rp.is_empty(),
            Self::NameTokens(tp) | Self::PathTokens(tp) => tp.is_empty(),
            Self::Composite(cp) => cp.is_empty(),
            Self::None => true,
        }
    }

    /// whether the scores are more than just 0 or 1.
    /// When it's the case, the tree builder will look for more matching results
    /// in order to select the best ones.
    pub fn has_real_scores(&self) -> bool {
        match self {
            Self::NameExact(_) | Self::NameFuzzy(_) => true,
            Self::PathExact(_) | Self::PathFuzzy(_) => true,
            Self::Composite(cp) => cp.has_real_scores(),
            _ => false,
        }
    }

}

