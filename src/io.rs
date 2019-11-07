/// This module deals is where is defined whether broot
/// writes on stdout, on stderr or elsewhere.


/// the type used by all GUI writing functions
pub type W = std::io::Stderr;

/// return the writer used by the application
pub fn writer() -> W {
    std::io::stderr()
}
