use crate::config::Config;
use crate::models::SessionInfo;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_db_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let db_dir = home.join(".pi").join("agent");
    fs::create_dir_all(&db_dir).map_err(|e| format!("Failed to create db dir: {}", e))?;
    Ok(db_dir.join("sessions.db"))
}

pub fn init_db() -> Result<Connection, String> {
    let config = Config::load_config().unwrap_or_default();
    init_db_with_config(&config)
}

pub fn init_db_with_config(config: &Config) -> Result<Connection, String> {
    let db_path = get_db_path()?;
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            cwd TEXT NOT NULL,
            name TEXT,
            created TEXT NOT NULL,
            modified TEXT NOT NULL,
            file_modified TEXT NOT NULL,
            message_count INTEGER NOT NULL,
            first_message TEXT,
            all_messages_text TEXT,
            cached_at TEXT NOT NULL,
            access_count INTEGER DEFAULT 0,
            last_accessed TEXT
        )",
        [],
    ).map_err(|e| format!("Failed to create table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_modified ON sessions(modified DESC)",
        [],
    ).map_err(|e| format!("Failed to create index idx_modified: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cwd ON sessions(cwd)",
        [],
    ).map_err(|e| format!("Failed to create index idx_cwd: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_file_modified ON sessions(file_modified)",
        [],
    ).map_err(|e| format!("Failed to create index idx_file_modified: {}", e))?;

    if config.enable_fts5 {
        init_fts5(&conn)?;
    }

    Ok(conn)
}

fn init_fts5(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(
            path UNINDEXED,
            cwd,
            name,
            first_message,
            all_messages_text,
            content='sessions',
            content_rowid='rowid'
        )",
        [],
    ).map_err(|e| format!("Failed to create FTS5 table: {}", e))?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS sessions_ai AFTER INSERT ON sessions BEGIN
            INSERT INTO sessions_fts(rowid, path, cwd, name, first_message, all_messages_text)
            VALUES (NEW.rowid, NEW.path, NEW.cwd, NEW.name, NEW.first_message, NEW.all_messages_text);
        END",
        [],
    ).map_err(|e| format!("Failed to create FTS5 insert trigger: {}", e))?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS sessions_ad AFTER DELETE ON sessions BEGIN
            INSERT INTO sessions_fts(sessions_fts, rowid, path, cwd, name, first_message, all_messages_text)
            VALUES ('delete', OLD.rowid, OLD.path, OLD.cwd, OLD.name, OLD.first_message, OLD.all_messages_text);
        END",
        [],
    ).map_err(|e| format!("Failed to create FTS5 delete trigger: {}", e))?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS sessions_au AFTER UPDATE ON sessions BEGIN
            INSERT INTO sessions_fts(sessions_fts, rowid, path, cwd, name, first_message, all_messages_text)
            VALUES ('delete', OLD.rowid, OLD.path, OLD.cwd, OLD.name, OLD.first_message, OLD.all_messages_text);
            INSERT INTO sessions_fts(rowid, path, cwd, name, first_message, all_messages_text)
            VALUES (NEW.rowid, NEW.path, NEW.cwd, NEW.name, NEW.first_message, NEW.all_messages_text);
        END",
        [],
    ).map_err(|e| format!("Failed to create FTS5 update trigger: {}", e))?;

    Ok(())
}

pub fn upsert_session(conn: &Connection, session: &SessionInfo, file_modified: DateTime<Utc>) -> Result<(), String> {
    conn.execute(
        "INSERT INTO sessions (id, path, cwd, name, created, modified, file_modified, message_count, first_message, all_messages_text, cached_at, access_count, last_accessed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 0, NULL)
         ON CONFLICT(path) DO UPDATE SET
            modified = excluded.modified,
            file_modified = excluded.file_modified,
            message_count = excluded.message_count,
            first_message = excluded.first_message,
            all_messages_text = excluded.all_messages_text,
            cached_at = excluded.cached_at",
        params![
            &session.id,
            &session.path,
            &session.cwd,
            &session.name,
            &session.created.to_rfc3339(),
            &session.modified.to_rfc3339(),
            &file_modified.to_rfc3339(),
            session.message_count as i64,
            &session.first_message,
            &session.all_messages_text,
            &Utc::now().to_rfc3339(),
        ],
    ).map_err(|e| format!("Failed to upsert session: {}", e))?;

    Ok(())
}

pub fn get_session(conn: &Connection, path: &str) -> Result<Option<SessionInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text
         FROM sessions WHERE path = ?"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let session = stmt.query_row(params![path], |row| {
        Ok(SessionInfo {
            path: row.get(1)?,
            id: row.get(0)?,
            cwd: row.get(2)?,
            name: row.get(3)?,
            created: parse_timestamp(&row.get::<_, String>(4)?),
            modified: parse_timestamp(&row.get::<_, String>(5)?),
            message_count: row.get(6)?,
            first_message: row.get(7)?,
            all_messages_text: row.get(8)?,
        })
    }).ok();

    if session.is_some() {
        conn.execute(
            "UPDATE sessions SET access_count = access_count + 1, last_accessed = ? WHERE path = ?",
            params![Utc::now().to_rfc3339(), path],
        ).ok();
    }

    Ok(session)
}

pub fn get_all_sessions(conn: &Connection) -> Result<Vec<SessionInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text
         FROM sessions ORDER BY modified DESC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let sessions = stmt.query_map([], |row| {
        Ok(SessionInfo {
            path: row.get(1)?,
            id: row.get(0)?,
            cwd: row.get(2)?,
            name: row.get(3)?,
            created: parse_timestamp(&row.get::<_, String>(4)?),
            modified: parse_timestamp(&row.get::<_, String>(5)?),
            message_count: row.get(6)?,
            first_message: row.get(7)?,
            all_messages_text: row.get(8)?,
        })
    }).map_err(|e| format!("Failed to query sessions: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect sessions: {}", e))?;

    Ok(sessions)
}

pub fn get_sessions_modified_after(conn: &Connection, cutoff: DateTime<Utc>) -> Result<Vec<SessionInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text
         FROM sessions WHERE modified > ? ORDER BY modified DESC"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let sessions = stmt.query_map(params![cutoff.to_rfc3339()], |row| {
        Ok(SessionInfo {
            path: row.get(1)?,
            id: row.get(0)?,
            cwd: row.get(2)?,
            name: row.get(3)?,
            created: parse_timestamp(&row.get::<_, String>(4)?),
            modified: parse_timestamp(&row.get::<_, String>(5)?),
            message_count: row.get(6)?,
            first_message: row.get(7)?,
            all_messages_text: row.get(8)?,
        })
    }).map_err(|e| format!("Failed to query sessions: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect sessions: {}", e))?;

    Ok(sessions)
}

pub fn get_cached_file_modified(conn: &Connection, path: &str) -> Result<Option<DateTime<Utc>>, String> {
    let mut stmt = conn.prepare(
        "SELECT file_modified FROM sessions WHERE path = ?"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let result = stmt.query_row(params![path], |row| {
        Ok(parse_timestamp(&row.get::<_, String>(0)?))
    }).ok();

    Ok(result)
}

pub fn delete_session(conn: &Connection, path: &str) -> Result<(), String> {
    conn.execute("DELETE FROM sessions WHERE path = ?", params![path])
        .map_err(|e| format!("Failed to delete session: {}", e))?;
    Ok(())
}

pub fn get_session_count(conn: &Connection) -> Result<usize, String> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count sessions: {}", e))?;
    Ok(count as usize)
}

pub fn vacuum(conn: &Connection) -> Result<(), String> {
    conn.execute("VACUUM", [])
        .map_err(|e| format!("Failed to vacuum database: {}", e))?;
    Ok(())
}

pub fn cleanup_missing_files(conn: &Connection) -> Result<usize, String> {
    let mut stmt = conn.prepare(
        "SELECT path FROM sessions"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let paths: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| format!("Failed to query paths: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect paths: {}", e))?;

    let mut deleted = 0;
    for path in paths {
        if !Path::new(&path).exists() {
            delete_session(conn, &path)?;
            deleted += 1;
        }
    }

    Ok(deleted)
}

pub fn preload_recent_sessions(conn: &Connection, count: usize) -> Result<Vec<SessionInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text
         FROM sessions
         ORDER BY last_accessed DESC, access_count DESC, modified DESC
         LIMIT ?"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let sessions = stmt.query_map(params![count as i64], |row| {
        Ok(SessionInfo {
            path: row.get(1)?,
            id: row.get(0)?,
            cwd: row.get(2)?,
            name: row.get(3)?,
            created: parse_timestamp(&row.get::<_, String>(4)?),
            modified: parse_timestamp(&row.get::<_, String>(5)?),
            message_count: row.get(6)?,
            first_message: row.get(7)?,
            all_messages_text: row.get(8)?,
        })
    }).map_err(|e| format!("Failed to query sessions: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect sessions: {}", e))?;

    Ok(sessions)
}

pub fn search_fts5(conn: &Connection, query: &str, limit: usize) -> Result<Vec<String>, String> {
    let mut stmt = conn.prepare(
        "SELECT path FROM sessions_fts
         WHERE sessions_fts MATCH ?
         ORDER BY rank
         LIMIT ?"
    ).map_err(|e| format!("Failed to prepare FTS5 statement: {}", e))?;

    let paths: Vec<String> = stmt.query_map(params![query, limit as i64], |row| row.get(0))
        .map_err(|e| format!("Failed to query FTS5: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect FTS5 results: {}", e))?;

    Ok(paths)
}

pub fn optimize_database(conn: &Connection) -> Result<(), String> {
    vacuum(conn)?;
    conn.execute("ANALYZE", [])
        .map_err(|e| format!("Failed to analyze database: {}", e))?;
    Ok(())
}

fn parse_timestamp(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}