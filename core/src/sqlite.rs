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
    // Any file opens; a file that is not a database fails on the first
    // query, run even when `tables` is empty.
    let marks = vec!["?"; tables.len()].join(", ");
    let found: i64 = conn
        .query_row(
            &format!(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name IN ({marks})"
            ),
            rusqlite::params_from_iter(tables),
            |r| r.get(0),
        )
        .map_err(not_a_log)?;
    if usize::try_from(found) != Ok(tables.len()) {
        return Err(Error::NotADiveLog(path.to_path_buf()));
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
    use tempfile::TempDir;

    /// Names of the files in `dir`.
    fn files(dir: &Path) -> Vec<String> {
        let mut names: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    fn dives(conn: &Connection) -> i64 {
        conn.query_row("SELECT count(*) FROM dives", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn opens_a_closed_wal_database_immutable_and_an_open_one_normally() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("log.sqlite");
        Connection::open(&path)
            .unwrap()
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 CREATE TABLE dives (n);
                 INSERT INTO dives VALUES (1);",
            )
            .unwrap();
        // Closed: no -wal file. Read immutable, creating none.
        assert!(is_closed_wal(&path));
        let before = files(dir.path());
        assert_eq!(before.len(), 1, "{before:?}");
        let log = open_read_only(&path, &["dives"]).unwrap();
        assert_eq!(dives(&log), 1);
        drop(log);
        assert_eq!(files(dir.path()), before, "file created next to it");

        // The application open, a dive still in its -wal file only: read
        // normally, so that the dive is there.
        let app = Connection::open(&path).unwrap();
        app.execute_batch(
            "PRAGMA wal_autocheckpoint = 0;
             INSERT INTO dives VALUES (2);",
        )
        .unwrap();
        assert!(!is_closed_wal(&path));
        let log = open_read_only(&path, &["dives"]).unwrap();
        assert_eq!(dives(&log), 2);
        drop(log);
        drop(app);

        // Not in WAL mode: opened normally.
        let journal = dir.path().join("journal.sqlite");
        Connection::open(&journal)
            .unwrap()
            .execute_batch("CREATE TABLE dives (n);")
            .unwrap();
        assert!(!is_closed_wal(&journal));
        assert_eq!(dives(&open_read_only(&journal, &["dives"]).unwrap()), 0);
    }

    #[test]
    fn needs_every_table_and_a_database_even_without_tables() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("log.sqlite");
        Connection::open(&path)
            .unwrap()
            .execute_batch("CREATE TABLE dives (n); CREATE TABLE sites (n);")
            .unwrap();
        assert!(open_read_only(&path, &["dives", "sites"]).is_ok());
        assert!(open_read_only(&path, &[]).is_ok());
        assert!(matches!(
            open_read_only(&path, &["dives", "samples"]),
            Err(Error::NotADiveLog(_))
        ));

        let text = dir.path().join("notes.txt");
        std::fs::write(
            &text,
            "not a database, but long enough to look like a header",
        )
        .unwrap();
        assert!(matches!(
            open_read_only(&text, &[]),
            Err(Error::NotADiveLog(_))
        ));
        assert!(matches!(
            open_read_only(&dir.path().join("none.sqlite"), &[]),
            Err(Error::NotFound(_))
        ));
    }

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
