
use {
    crate::{
        app::{
            AppContext,
            AppStateCmdResult,
        },
        errors::ProgramError,
        screens::Screen,
    },
    super::{
        Verb,
        VerbInvocation,
    },
};

pub trait VerbExecutor {
    fn execute_verb(
        &mut self,
        verb: &Verb,
        invocation: Option<&VerbInvocation>,
        screen: &mut Screen,
        con: &AppContext,
    ) -> Result<AppStateCmdResult, ProgramError>;
}
