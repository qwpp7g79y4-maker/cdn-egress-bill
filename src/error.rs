use std::fmt;

/// Bare message of a byte-count overflow while pricing; the CLI adds the log paths.
pub const OVERFLOW: &str = "overflow";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::report::visible(&self.message))
    }
}

impl std::error::Error for Error {}
