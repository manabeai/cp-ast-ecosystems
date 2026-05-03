use std::fmt;

/// Errors during AST ↔ JSON conversion.
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// Invalid decimal ID string.
    InvalidId(String),
    /// Unknown enum variant tag.
    UnknownVariant {
        type_name: &'static str,
        value: String,
    },
    /// JSON parse/stringify error.
    Json(String),
    /// Share state base64 decoding error.
    Base64(String),
    /// Share state gzip decoding error.
    Gzip(String),
    /// Schema version mismatch.
    UnsupportedVersion(u32),
    /// Arena slot ID doesn't match its index.
    IdIndexMismatch { expected: u64, actual: u64 },
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId(s) => write!(f, "invalid ID string: {s:?}"),
            Self::UnknownVariant { type_name, value } => {
                write!(f, "unknown {type_name} variant: {value:?}")
            }
            Self::Json(msg) => write!(f, "JSON error: {msg}"),
            Self::Base64(msg) => write!(f, "base64 decode error: {msg}"),
            Self::Gzip(msg) => write!(f, "gzip decode error: {msg}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported schema version: {v}"),
            Self::IdIndexMismatch { expected, actual } => {
                write!(
                    f,
                    "arena slot ID mismatch: expected {expected}, got {actual}"
                )
            }
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<serde_json::Error> for ConversionError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err.to_string())
    }
}
