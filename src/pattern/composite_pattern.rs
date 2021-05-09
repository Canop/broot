use {
    super::*,
    crate::{
        content_search::ContentMatch,
    },
    bet::*,
    std::path::Path,
};

/// A pattern composing other ones with operators
#[derive(Debug, Clone)]
pub struct CompositePattern {
    pub expr: BeTree<PatternOperator, Pattern>,
}

impl CompositePattern {
    pub fn new(expr: BeTree<PatternOperator, Pattern>) -> Self {
        Self { expr }
    }

    pub fn score_of_string(&self, candidate: &str) -> Option<i32> {
        use PatternOperator::*;
        let composite_result: Option<Option<i32>> = self.expr.eval(
            // score evaluation
            |pat| pat.score_of_string(candidate),
            // operator
            |op, a, b| {
                match (op, a, b) {
                    (And, None, _) => None, // normally not called due to short-circuit
                    (And, Some(sa), Some(Some(sb))) => Some(sa + sb),
                    (Or, None, Some(Some(sb))) => Some(sb),
                    (Or, Some(sa), Some(None)) => Some(sa),
                    (Or, Some(sa), Some(Some(sb))) => Some(sa + sb),
                    (Not, Some(_), _) => None,
                    (Not, None, _) => Some(1),
                    _ => None,
                }
            },
            // short-circuit. We don't short circuit on 'or' because
            // we want to use both scores
            |op, a| match (op, a) {
                (And, None) => true,
                _ => false,
            },
        );
        composite_result
            .unwrap_or_else(||{
                warn!("unexpectedly missing result ");
                None
            })
    }

    pub fn score_of(&self, candidate: Candidate) -> Option<i32> {
        use PatternOperator::*;
        let composite_result: Option<Option<i32>> = self.expr.eval(
            // score evaluation
            |pat| pat.score_of(candidate),
            // operator
            |op, a, b| {
                match (op, a, b) {
                    (And, None, _) => None, // normally not called due to short-circuit
                    (And, Some(sa), Some(Some(sb))) => Some(sa + sb),
                    (Or, None, Some(Some(sb))) => Some(sb),
                    (Or, Some(sa), Some(None)) => Some(sa),
                    (Or, Some(sa), Some(Some(sb))) => Some(sa + sb),
                    (Not, Some(_), _) => None,
                    (Not, None, _) => Some(1),
                    _ => None,
                }
            },
            // short-circuit. We don't short circuit on 'or' because
            // we want to use both scores
            |op, a| match (op, a) {
                (And, None) => true,
                _ => false,
            },
        );
        composite_result
            .unwrap_or_else(||{
                warn!("unexpectedly missing result ");
                None
            })
    }

    pub fn search_string(&self, candidate: &str) -> Option<NameMatch> {
        // an ideal algorithm would call score_of on patterns when the object is different
        // to deal with exclusions but I'll start today with something simpler
        use PatternOperator::*;
        let composite_result: Option<Option<NameMatch>> = self.expr.eval(
            // score evaluation
            |pat| pat.search_string(candidate),
            // operator
            |op, a, b| match (op, a, b) {
                (Not, Some(_), _) => None,
                (_, Some(ma), _) => Some(ma),
                (_, None, Some(omb)) => omb,
                _ => None,
            },
            |op, a| match (op, a) {
                (Or, Some(_)) => true,
                _ => false,
            },
        );
        // it's possible we didn't find a result because the composition
        composite_result
            .unwrap_or_else(||{
                warn!("unexpectedly missing result ");
                None
            })
    }

    pub fn search_content(
        &self,
        candidate: &Path,
        desired_len: usize, // available space for content match display
    ) -> Option<ContentMatch> {
        use PatternOperator::*;
        let composite_result: Option<Option<ContentMatch>> = self.expr.eval(
            // score evaluation
            |pat| pat.search_content(candidate, desired_len),
            // operator
            |op, a, b| match (op, a, b) {
                (Not, Some(_), _) => None,
                (_, Some(ma), _) => Some(ma),
                (_, None, Some(omb)) => omb,
                _ => None,
            },
            |op, a| match (op, a) {
                (Or, Some(_)) => true,
                _ => false,
            },
        );
        composite_result
            .unwrap_or_else(||{
                warn!("unexpectedly missing result ");
                None
            })
    }

    pub fn get_match_line_count(
        &self,
        candidate: &Path,
    ) -> Option<usize> {
        use PatternOperator::*;
        let composite_result: Option<Option<usize>> = self.expr.eval(
            // score evaluation
            |pat| pat.get_match_line_count(candidate),
            // operator
            |op, a, b| match (op, a, b) {
                (Not, Some(_), _) => None,
                (_, Some(ma), _) => Some(ma),
                (_, None, Some(omb)) => omb,
                _ => None,
            },
            |op, a| match (op, a) {
                (Or, Some(_)) => true,
                _ => false,
            },
        );
        composite_result
            .unwrap_or_else(||{
                warn!("unexpectedly missing result ");
                None
            })
    }

    pub fn has_real_scores(&self) -> bool {
        self.expr.iter_atoms()
            .fold(false, |r, p| match p {
                Pattern::NameFuzzy(_) | Pattern::PathFuzzy(_) => true,
                _ => r,
            })
    }

    pub fn is_empty(&self) -> bool {
        let is_not_empty = self.expr.iter_atoms()
            .any(|p| p.is_some());
        !is_not_empty
    }

}
