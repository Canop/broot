use {
    super::*,
    crate::{
        content_search::ContentMatch,
        errors::PatternError,
    },
    bet::*,
    std::{
        path::Path,
    },
};

/// A pattern composing other ones with operators
#[derive(Debug, Clone)]
pub struct CompositePattern {
    pub expr: BeTree<PatternOperator, Pattern>,
}

impl CompositePattern {
    pub fn new(expr: BeTree<PatternOperator, Pattern>) -> Self {
        Self {
            expr
        }
    }

    pub fn score_of_string(&self, candidate: &str) -> Option<i32> {
        use PatternOperator::*;
        let composite_result: Result<Option<Option<i32>>, PatternError> = self.expr.eval(
            // score evaluation
            |pat| Ok(pat.score_of_string(candidate)),
            // operator
            |op, a, b| Ok(match (op, a, b) {
                (And, None, _) => None, // normally not called due to short-circuit
                (And, Some(sa), Some(Some(sb))) => Some(sa+sb),
                (Or, None, Some(Some(sb))) => Some(sb),
                (Or, Some(sa), Some(None)) => Some(sa),
                (Or, Some(sa), Some(Some(sb))) => Some(sa+sb),
                (Not, Some(_), _) => None,
                (Not, None, _) => Some(1),
                _ => None,
            }),
            // short-circuit. We don't short circuit on 'or' because
            // we want to use both scores
            |op, a| match (op, a) {
                (And, None) => true,
                _ => false,
            }
        );
        match composite_result {
            Err(e) => {
                warn!("unexpected error while evaluating composite: {:?}", e);
                None
            }
            Ok(Some(r)) => r,
            Ok(None) => {
                warn!("unexpectedly missing result ");
                None
            }
        }
    }

    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        use PatternOperator::*;
        let composite_result: Result<Option<Option<i32>>, PatternError> = self.expr.eval(
            // score evaluation
            |pat| Ok(pat.score_of(candidate)),
            // operator
            |op, a, b| Ok(match (op, a, b) {
                (And, None, _) => None, // normally not called due to short-circuit
                (And, Some(sa), Some(Some(sb))) => Some(sa+sb),
                (Or, None, Some(Some(sb))) => Some(sb),
                (Or, Some(sa), Some(None)) => Some(sa),
                (Or, Some(sa), Some(Some(sb))) => Some(sa+sb),
                (Not, Some(_), _) => None,
                (Not, None, _) => Some(1),
                _ => None,
            }),
            // short-circuit. We don't short circuit on 'or' because
            // we want to use both scores
            |op, a| match (op, a) {
                (And, None) => true,
                _ => false,
            }
        );
        match composite_result {
            Err(e) => {
                warn!("unexpected error while evaluating composite: {:?}", e);
                None
            }
            Ok(Some(r)) => r,
            Ok(None) => {
                warn!("unexpectedly missing result ");
                None
            }
        }
    }

    pub fn search_string(
        &self,
        candidate: &str,
    ) -> Option<NameMatch> {
        // an ideal algorithm would call score_of on patterns when the object is different
        // to deal with exclusions but I'll start today with something simpler
        use PatternOperator::*;
        let composite_result: Result<Option<Option<NameMatch>>, PatternError> = self.expr.eval(
            // score evaluation
            |pat| Ok(pat.search_string(candidate)),
            // operator
            |op, a, b| Ok(match (op, a, b) {
                (Not, Some(_), _) => None,
                (_, Some(ma), _) => Some(ma),
                (_, None, Some(omb)) => omb,
                _ => None,
            }),
            |op, a| match (op, a) {
                (Or, Some(_)) => true,
                _ => false,
            }
        );
        // it's possible we didn't find a result because the composition
        match composite_result {
            Err(e) => {
                warn!("unexpected error while evaluating composite: {:?}", e);
                None
            }
            Ok(Some(r)) => r,
            Ok(None) => {
                warn!("unexpectedly missing result ");
                None
            }
        }
    }

    pub fn search_content(
        &self,
        candidate: &Path,
        desired_len: usize, // available space for content match display
    ) -> Option<ContentMatch> {
        use PatternOperator::*;
        let composite_result: Result<Option<Option<ContentMatch>>, PatternError> = self.expr.eval(
            // score evaluation
            |pat| Ok(pat.search_content(candidate, desired_len)),
            // operator
            |op, a, b| Ok(match (op, a, b) {
                (Not, Some(_), _) => None,
                (_, Some(ma), _) => Some(ma),
                (_, None, Some(omb)) => omb,
                _ => None,
            }),
            |op, a| match (op, a) {
                (Or, Some(_)) => true,
                _ => false,
            }
        );
        match composite_result {
            Err(e) => {
                warn!("unexpected error while evaluating composite: {:?}", e);
                None
            }
            Ok(Some(r)) => r,
            Ok(None) => {
                warn!("unexpectedly missing result ");
                None
            }
        }
    }

    pub fn has_real_scores(&self) -> bool {
        self.expr.iter_atoms()
            .fold(false, |r, p| match p {
                Pattern::NameFuzzy(_) | Pattern::PathFuzzy(_) => true,
                _ => r,
            })
    }

}
