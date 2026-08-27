use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum AppError {
    Command(io::Error),
    Parse(String),
    Unity(String),
    Message(String),
}

impl Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(err) => write!(f, "{err}"),
            Self::Parse(msg) | Self::Unity(msg) | Self::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::Command(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value.to_string())
    }
}
