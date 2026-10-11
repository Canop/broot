use std::io;

#[derive(thiserror::Error, Debug)]
// Note: the error messages are markdown, rendered with termimad by [`ProgramError::report`] or in the status line
pub enum ShellInstallError {
    #[error("IO Error {source} on {when}")]
    Io { source: io::Error, when: String },
}
impl ShellInstallError {
    pub fn is_permission_denied(&self) -> bool {
        match self {
            Self::Io { source, .. } => {
                if source.kind() == io::ErrorKind::PermissionDenied {
                    true
                } else {
                    cfg!(windows) && source.raw_os_error().unwrap_or(0) == 1314
                }
            }
        }
    }
}
pub trait IoToShellInstallError<Ok> {
    fn context(
        self,
        f: &dyn Fn() -> String,
    ) -> Result<Ok, ShellInstallError>;
}
impl<Ok> IoToShellInstallError<Ok> for Result<Ok, io::Error> {
    fn context(
        self,
        f: &dyn Fn() -> String,
    ) -> Result<Ok, ShellInstallError> {
        self.map_err(|source| ShellInstallError::Io { source, when: f() })
    }
}
