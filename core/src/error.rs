use std::fmt;
use std::path::PathBuf;

use serde::Serialize;

/// Errors a dive-log reader can report. Serialized to the UI as
/// `{ "kind": "...", "detail": "..." }` so the UI can pick its own message.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "detail", rename_all = "camelCase")]
pub enum Error {
    /// Nothing at the given path.
    NotFound(PathBuf),
    /// The file exists but is not a database of the expected application.
    NotADiveLog(PathBuf),
    /// A dive log saved by a version of the application too old to be read;
    /// opening and saving it again in that application updates it.
    OldFormat(PathBuf),
    /// The folder holds several dive logs and none is the current one.
    SeveralLogs(PathBuf),
    /// The database could not be read (locked, corrupt, not downloaded…).
    Database(String),
    /// A dive with this id does not exist in the source.
    UnknownDive(String),
    /// A dive profile could not be decoded.
    BadProfile(String),
    /// libdivecomputer could not decode the dive computer's data.
    Decoder(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound(p) => write!(f, "no dive log at {}", p.display()),
            Error::NotADiveLog(p) => write!(f, "{} is not a dive log", p.display()),
            Error::OldFormat(p) => write!(f, "{} uses an old format", p.display()),
            Error::SeveralLogs(p) => write!(f, "{} holds several dive logs", p.display()),
            Error::Database(m) => write!(f, "database error: {m}"),
            Error::UnknownDive(id) => write!(f, "unknown dive {id}"),
            Error::BadProfile(m) => write!(f, "unreadable dive profile: {m}"),
            Error::Decoder(m) => write!(f, "dive computer data: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Database(e.to_string())
    }
}
