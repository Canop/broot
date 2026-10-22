mod coarity;
//mod arg_def;
mod exec_pattern;
mod exec_arg_def;
mod execution_builder;
#[cfg(all(test, unix))]
mod execution_builder_tests;
mod external_execution;
mod external_execution_mode;
mod file_type_condition;
mod internal;
mod internal_execution;
pub mod internal_focus;
pub mod internal_path;
pub mod internal_select;
mod invocation_arg_def;
mod invocation_parser;
mod sequence_execution;
mod standard_arg;
mod verb;
mod verb_description;
mod verb_execution;
mod verb_invocation;
mod verb_store;
mod write;

use lazy_regex::*;
pub use {
    coarity::*,
    //arg_def::*,
    exec_arg_def::*,
    exec_pattern::*,
    execution_builder::*,
    external_execution::*,
    external_execution_mode::ExternalExecutionMode,
    file_type_condition::*,
    internal::Internal,
    internal_execution::InternalExecution,
    invocation_arg_def::*,
    invocation_parser::InvocationParser,
    once_cell::sync::Lazy,
    sequence_execution::SequenceExecution,
    standard_arg::StandardArg,
    verb::Verb,
    verb_description::VerbDescription,
    verb_execution::VerbExecution,
    verb_invocation::*,
    verb_store::{
        PrefixSearchResult,
        VerbStore,
    },
    write::*,
};


pub type VerbId = usize;

/// A `{name}` or `{name:flags}` group in a verb definition string.
///
/// The meaning of the flags depends on whether the group is in an invocation
/// pattern (see `InvocationArgDef`) or in an execution pattern (see `ExecArgDef`).
pub static ARG_DEF_GROUP: Lazy<Regex> = lazy_regex!(r"\{([^{}:]+)(?::([^{}:]+))?\}");
