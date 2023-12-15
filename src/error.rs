use std::io;

use thiserror::Error;

use crate::bencode_format::ParseError;

#[derive(Debug, Error)]
pub enum TorrentError {
    #[error("HTTP: {0}")]
    Http(String),

    #[error("I/O: {0}")]
    Io(String),

    #[error("Bencode: {0}")]
    Bencode(String),

    #[error("JSON: {0}")]
    Json(String),

    #[error("Invalid message ID")]
    InvalidMessageId,
}

impl From<reqwest::Error> for TorrentError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
    }
}

impl From<io::Error> for TorrentError {
    fn from(err: io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<ParseError> for TorrentError {
    fn from(err: ParseError) -> Self {
        Self::Bencode(err.to_string())
    }
}

impl From<serde_json::Error> for TorrentError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}
