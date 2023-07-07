
mod candidate;
mod composite_pattern;
mod content_pattern;
mod content_regex_pattern;
mod exact_pattern;
mod fuzzy_pattern;
mod input_pattern;
mod name_match;
mod operator;
mod pattern;
mod pattern_object;
mod pattern_parts;
mod pos;
mod regex_pattern;
mod search_mode;
mod tok_pattern;

pub use {
    candidate::Candidate,
    composite_pattern::CompositePattern,
    content_pattern::ContentExactPattern,
    content_regex_pattern::ContentRegexPattern,
    exact_pattern::ExactPattern,
    fuzzy_pattern::FuzzyPattern,
    input_pattern::InputPattern,
    name_match::NameMatch,
    pattern::Pattern,
    pattern_object::PatternObject,
    pattern_parts::PatternParts,
    pos::*,
    operator::PatternOperator,
    regex_pattern::RegexPattern,
    search_mode::*,
    tok_pattern::*,
};

use {
    crate::errors::PatternError,
    lazy_regex::regex,
};

pub fn build_regex(pat: &str, flags: &str) -> Result<regex::Regex, PatternError> {
    let mut builder = regex::RegexBuilder::new(pat);
    for c in flags.chars() {
        match c {
            'i' => {
                builder.case_insensitive(true);
            }
            'U' => {
                builder.swap_greed(true);
            }
            _ => {
                return Err(PatternError::UnknownRegexFlag { bad: c });
            }
        }
    }
    Ok(builder.build()?)
}
