use thiserror::Error;

use crate::bencode_format::ParseError;

#[derive(Debug, Error)]
pub enum TorrentError {
    #[error("HTTP: {0}")]
    Http(String),

    #[error("Bencode: {0}")]
    Bencode(String),

    #[error("JSON: {0}")]
    Json(String),
}

impl From<reqwest::Error> for TorrentError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err.to_string())
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
