//! Opening a dive application's SQLite database read-only, without changing
//! or creating any file next to it. Shared by every reader of a SQLite log.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use crate::{Error, Result};

/// Opens the database at `path` read-only; the application may stay open
/// meanwhile. `tables` are the tables every database of the application
/// has: a file without them is not one of its logs.
///
/// The database and its `-wal` file are never changed. While the
/// application is closed, nothing next to the database is created or changed
/// either; while it is open, SQLite updates its `-shm` index, as it does for
/// any reader.
pub(crate) fn open_read_only(path: &Path, tables: &[&str]) -> Result<Connection> {
    if !path.is_file() {
        return Err(Error::NotFound(path.to_path_buf()));
    }
    let not_a_log = |_| Error::NotADiveLog(path.to_path_buf());
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = match immutable_uri(path).filter(|_| is_closed_wal(path)) {
        Some(uri) => Connection::open_with_flags(uri, flags | OpenFlags::SQLITE_OPEN_URI),
        None => Connection::open_with_flags(path, flags),
    }
    .map_err(not_a_log)?;
    // Any file opens; a file that is not a database fails on the first query.
    for table in tables {
        let found: bool = conn
            .query_row(
                "SELECT count(*) = 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |r| r.get(0),
            )
            .map_err(not_a_log)?;
        if !found {
            return Err(Error::NotADiveLog(path.to_path_buf()));
        }
    }
    Ok(conn)
}

/// A database in WAL mode (as Core Data and many applications write them)
/// whose `-wal` file is missing or empty: every change is in the database
/// file itself. Opened normally, even read-only, SQLite would create `-wal`
/// and `-shm` files next to it, or change a `-shm` file left there; opened
/// "immutable", it creates and changes nothing, and misses nothing. When the
/// `-wal` file holds anything (the application open), it may hold dives not
/// yet copied into the database file, so the database is opened normally.
///
/// Opened immutable, the database is read as it stands when the connection
/// opens: should the application start and copy its changes into the file
/// meanwhile, a read could miss them or fail. Connections last one command
/// of the app, and the next command reads the database again.
fn is_closed_wal(path: &Path) -> bool {
    // Bytes 18 and 19 of the SQLite header are 2 in WAL mode.
    let mut header = [0u8; 20];
    let wal_mode = File::open(path)
        .and_then(|mut f| f.read_exact(&mut header))
        .is_ok_and(|()| header[18] == 2 && header[19] == 2);
    let mut wal = path.as_os_str().to_owned();
    wal.push("-wal");
    wal_mode && !std::fs::metadata(&wal).is_ok_and(|m| m.len() > 0)
}

/// SQLite `file:` URI opening `path` immutable; `None` for a path SQLite's
/// URIs cannot name (not Unicode; on Windows, not on a drive letter).
fn immutable_uri(path: &Path) -> Option<String> {
    let path = std::path::absolute(path).ok()?;
    let text = path.to_str()?;
    let mut uri = String::from("file:");
    if cfg!(windows) {
        // "C:\Logs\MacDive.sqlite" is written "file:/C:/Logs/MacDive.sqlite".
        if text.as_bytes().get(1) != Some(&b':') {
            return None;
        }
        uri.push('/');
    }
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' => {
                uri.push(char::from(b));
            }
            b'\\' if cfg!(windows) => uri.push('/'),
            b':' if cfg!(windows) && i == 1 => uri.push(':'),
            _ => uri.push_str(&format!("%{b:02X}")),
        }
    }
    uri.push_str("?immutable=1");
    Some(uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(windows))]
    fn immutable_uris_escape_the_path() {
        assert_eq!(
            immutable_uri(Path::new("/Users/diver/Mac Dive ~ #1 100% été.sqlite")).as_deref(),
            Some(
                "file:/Users/diver/Mac%20Dive%20~%20%231%20100%25%20%C3%A9t%C3%A9.sqlite?immutable=1"
            )
        );
    }

    #[test]
    #[cfg(windows)]
    fn immutable_uris_escape_the_path() {
        assert_eq!(
            immutable_uri(Path::new(r"C:\Logs\Mac Dive #1.sqlite")).as_deref(),
            Some("file:/C:/Logs/Mac%20Dive%20%231.sqlite?immutable=1")
        );
        assert_eq!(
            immutable_uri(Path::new(r"\\server\share\MacDive.sqlite")),
            None
        );
    }
}
