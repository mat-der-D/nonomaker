use std::fmt;

pub enum CliError {
    Io(std::io::Error),
    Format(nonomaker_core::format::FormatError),
    Parse(String),
    Validation(String),
    ImageDecode(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(e) => write!(f, "io error: {e}"),
            CliError::Format(e) => write!(f, "format error: {e}"),
            CliError::Parse(msg) => write!(f, "parse error: {msg}"),
            CliError::Validation(msg) => write!(f, "validation error: {msg}"),
            CliError::ImageDecode(msg) => write!(f, "image decode error: {msg}"),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<nonomaker_core::format::FormatError> for CliError {
    fn from(e: nonomaker_core::format::FormatError) -> Self {
        CliError::Format(e)
    }
}
