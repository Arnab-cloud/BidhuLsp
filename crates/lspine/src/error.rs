use std::{error, fmt, io};

#[derive(Debug)]
pub enum LSPError {
    Io(io::Error),
    Json(serde_json::Error),
    Protocol(String),
    Other(String),
}

impl fmt::Display for LSPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO ERROR: {err}"),
            Self::Other(err) => write!(f, "ERROR: {err}"),
            Self::Json(err) => write!(f, "JSON ERROR: {err}"),
            Self::Protocol(err) => write!(f, "PROTOCOL ERROR: {err}"),
        }
    }
}

impl error::Error for LSPError {}

impl From<io::Error> for LSPError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for LSPError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
