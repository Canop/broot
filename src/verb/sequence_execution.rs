
use {
    crate::{
        command::Sequence,
    },
};

/// A verb execution definition based on a sequence
/// of commands
#[derive(Debug, Clone)]
pub struct SequenceExecution {

    pub sequence: Sequence,

}
