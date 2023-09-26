mod arg_def;
mod exec_pattern;
mod execution_builder;
mod external_execution;
mod external_execution_mode;
mod internal;
mod internal_execution;
pub mod internal_focus;
pub mod internal_select;
mod invocation_parser;
mod sequence_execution;
mod verb;
mod verb_description;
mod verb_execution;
mod verb_invocation;
mod verb_store;

pub use {
    arg_def::*,
    exec_pattern::*,
    execution_builder::ExecutionStringBuilder,
    external_execution::ExternalExecution,
    external_execution_mode::ExternalExecutionMode,
    internal::Internal,
    internal_execution::InternalExecution,
    invocation_parser::InvocationParser,
    once_cell::sync::Lazy,
    sequence_execution::SequenceExecution,
    verb::Verb,
    verb_description::VerbDescription,
    verb_execution::VerbExecution,
    verb_invocation::*,
    verb_store::{PrefixSearchResult, VerbStore},
};
use {
    lazy_regex::*,
};

/// the group you find in invocation patterns and execution patterns
pub static GROUP: Lazy<Regex> = lazy_regex!(r"\{([^{}:]+)(?::([^{}:]+))?\}");

pub type VerbId = usize;

pub fn str_has_selection_group(s: &str) -> bool {
    GROUP.find_iter(s)
        .any(|group| matches!(
            group.as_str(),
            "{file}" | "{file-name}" | "{parent}" | "{directory}",
        ))
}
pub fn str_has_other_panel_group(s: &str) -> bool {
    for group in GROUP.find_iter(s) {
        if group.as_str().starts_with("{other-panel-") {
            return true;
        }
    }
    false
}

