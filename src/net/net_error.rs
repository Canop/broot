use std::io;

#[derive(thiserror::Error, Debug)]
// Note: the error messages are markdown, rendered with termimad by [`ProgramError::report`] or in the status line
pub enum NetError {
    #[error("Can't open socket: `{path}` already exists - consider removing it")]
    SocketNotAvailable { path: String },
    #[error("a broot server with the name `{name}` is already running")]
    DuplicateServerName { name: String },
    #[error("error on the socket: {source}")]
    Io {
        #[from]
        source: io::Error,
    },
    #[error("invalid message received")]
    InvalidMessage,
}
