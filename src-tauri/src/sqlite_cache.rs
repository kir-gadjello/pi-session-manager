use crate::config::Config;
use crate::session_parser::SessionDetails;
use crate::models::SessionInfo;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result as SqliteResult, OptionalExtension};
use std::fs;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

pub fn get_db_path() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    let sessions_dir = home.join(".pi").join("agent").join("sessions");
    fs::create_dir_all(&sessions_dir).map_err(|e| format!("Failed to create sessions dir: {}", e))?;
    Ok(sessions_dir.join("sessions.db"))
}

pub fn init_db() -> Result<Connection, String> {
    let config = Config::load_config().unwrap_or_default();
    init_db_with_config(&config)
}

pub fn init_db_with_config(config: &Config) -> Result<Connection, String> {
    let db_path = get_db_path()?;
    
    match open_and_init_db(&db_path, config) {
        Ok(conn) => Ok(conn),
        Err(e) if e.contains("malformed") || e.contains("disk image") || e.contains("not a database") => {
            eprintln!("[Recovery] Database corrupted ({}). Deleting and recreating...", e);
            fs::remove_file(&db_path).map_err(|err| format!("Failed to delete corrupted DB: {}", err))?;
            open_and_init_db(&db_path, config)
        },
        Err(e) => Err(e),
    }
}

fn open_and_init_db(db_path: &Path, config: &Config) -> Result<Connection, String> {
    let conn = Connection::open(db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    // Enable WAL mode for better concurrency and reliability
    conn.prepare("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("Failed to set WAL mode: {}", e))?
        .query_row([], |_| Ok(()))
        .map_err(|e| format!("Failed to set WAL mode: {}", e))?;
    
    // Set synchronous mode (does not return a result row)
    conn.execute("PRAGMA synchronous=NORMAL;", [])
        .map_err(|e| format!("Failed to set synchronous mode: {}", e))?;
    
    // Enable foreign key constraints
    conn.execute("PRAGMA foreign_keys=ON;", [])
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

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
            user_messages_text TEXT,
            assistant_messages_text TEXT,
            last_message TEXT,
            last_message_role TEXT,
            cached_at TEXT NOT NULL,
            access_count INTEGER DEFAULT 0,
            last_accessed TEXT
        )",
        [],
    ).map_err(|e| format!("Failed to create table: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS session_details_cache (
            path TEXT PRIMARY KEY,
            file_modified TEXT NOT NULL,
            user_messages INTEGER NOT NULL,
            assistant_messages INTEGER NOT NULL,
            input_tokens INTEGER NOT NULL,
            output_tokens INTEGER NOT NULL,
            cache_read_tokens INTEGER NOT NULL,
            cache_write_tokens INTEGER NOT NULL,
            input_cost REAL NOT NULL,
            output_cost REAL NOT NULL,
            cache_read_cost REAL NOT NULL,
            cache_write_cost REAL NOT NULL,
            models_json TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("Failed to create table session_details_cache: {}", e))?;

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

    // Create favorites table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS favorites (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL CHECK(type IN ('session', 'project')),
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            added_at TEXT NOT NULL
        )",
        [],
    ).map_err(|e| format!("Failed to create favorites table: {}", e))?;

    // 迁移：添加 last_message 和 last_message_role 字段（如果不存在）
    conn.execute(
        "ALTER TABLE sessions ADD COLUMN last_message TEXT",
        [],
    ).ok(); // 忽略错误（字段可能已存在）

    conn.execute(
        "ALTER TABLE sessions ADD COLUMN last_message_role TEXT",
        [],
    ).ok(); // 忽略错误（字段可能已存在）

    conn.execute(
        "ALTER TABLE sessions ADD COLUMN user_messages_text TEXT",
        [],
    ).ok(); // 忽略错误

    conn.execute(
        "ALTER TABLE sessions ADD COLUMN assistant_messages_text TEXT",
        [],
    ).ok(); // 忽略错误

    // Create message_entries table for per-message FTS (avoids re-reading files)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message_entries (
            id TEXT PRIMARY KEY,
            session_path TEXT NOT NULL,
            role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY (session_path) REFERENCES sessions(path) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| format!("Failed to create message_entries table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_message_entries_session ON message_entries(session_path)",
        [],
    ).map_err(|e| format!("Failed to create index on message_entries: {}", e))?;

    // Migrate message_entries schema if outdated (missing columns)
    {
        let mut stmt = conn.prepare("PRAGMA table_info(message_entries)").map_err(|e| e.to_string())?;
        let me_columns: Vec<String> = stmt.query_map([], |row| row.get(1))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let required_me_columns = ["id", "session_path", "role", "content", "timestamp"];
        let me_has_all = required_me_columns.iter().all(|&col| me_columns.contains(&col.to_string()));
        if !me_has_all {
            eprintln!("[Migration] message_entries schema outdated (missing columns). Dropping and recreating...");
            // Drop FTS first because it depends on message_entries
            let _ = conn.execute("DROP TABLE IF EXISTS message_fts", []); // ignore error
            conn.execute("DROP TABLE IF EXISTS message_entries", []).map_err(|e| e.to_string())?;
            // Recreate message_entries with full schema
            conn.execute(
                "CREATE TABLE message_entries (
                    id TEXT PRIMARY KEY,
                    session_path TEXT NOT NULL,
                    role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
                    content TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    FOREIGN KEY (session_path) REFERENCES sessions(path) ON DELETE CASCADE
                )",
                [],
            ).map_err(|e| format!("Failed to recreate message_entries: {}", e))?;
            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_message_entries_session ON message_entries(session_path)",
                [],
            ).map_err(|e| format!("Failed to create index on message_entries: {}", e))?;
        }
    }

    if config.enable_fts5 {
        init_fts5(&conn)?;
        // Comprehensive schema check for message-level FTS
        ensure_message_fts_schema(&conn)?;
    }

    Ok(conn)
}

fn init_fts5(conn: &Connection) -> Result<(), String> {
    // Check if we need to upgrade FTS5 table
    let mut stmt = conn.prepare("PRAGMA table_info(sessions_fts)").map_err(|e| e.to_string())?;
    let columns: Vec<String> = stmt.query_map([], |row| row.get(1))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    if columns.is_empty() || !columns.contains(&"user_messages_text".to_string()) {
        full_rebuild_fts(conn)?;
    }

    Ok(())
}

fn ensure_message_fts_schema(conn: &Connection) -> Result<(), String> {
    // Check if message_entries table exists and has required columns
    let mut stmt = conn.prepare("PRAGMA table_info(message_entries)").map_err(|e| format!("Failed to query message_entries schema: {}", e))?;
    let me_columns: Vec<String> = stmt.query_map([], |row| row.get(1))
        .map_err(|e| format!("Failed to read message_entries columns: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect message_entries columns: {}", e))?;
    let required_me_columns = ["id", "session_path", "role", "content", "timestamp"];
    let me_has_all = required_me_columns.iter().all(|&col| me_columns.contains(&col.to_string()));
    
    if !me_has_all {
        error!("[Migration] message_entries schema incomplete. Has columns: {:?}. Recreating table...", me_columns);
        // Drop FTS first because it references message_entries
        let _ = conn.execute("DROP TABLE IF EXISTS message_fts", []); // ignore error
        conn.execute("DROP TABLE IF EXISTS message_entries", []).map_err(|e| format!("Failed to drop old message_entries: {}", e))?;
        // Recreate with full schema
        conn.execute(
            "CREATE TABLE message_entries (
                id TEXT PRIMARY KEY,
                session_path TEXT NOT NULL,
                role TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (session_path) REFERENCES sessions(path) ON DELETE CASCADE
            )",
            [],
        ).map_err(|e| format!("Failed to recreate message_entries: {}", e))?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_message_entries_session ON message_entries(session_path)",
            [],
        ).map_err(|e| format!("Failed to create index on message_entries: {}", e))?;
        info!("[Migration] Recreated message_entries table");
    } else {
        debug!("[Schema] message_entries columns OK: {:?}", me_columns);
    }
    
    // Ensure message_fts exists with correct schema (no triggers needed for content-bearing FTS5)
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='message_fts'")
        .map_err(|e| format!("Failed to check message_fts existence: {}", e))?;
    let fts_exists = stmt.query_row([], |row| Ok(row.get::<_, String>(0)? == "message_fts"))
        .unwrap_or(false);
    
    if !fts_exists {
        info!("[FTS] Creating message_fts virtual table");
        create_message_fts5(conn)?;
        // Rebuild index from existing message_entries
        conn.execute("INSERT INTO message_fts(message_fts) VALUES('rebuild')", [])
            .map_err(|e| format!("Failed to rebuild message_fts index: {}", e))?;
        info!("[FTS] Rebuilt message_fts index");
        return Ok(());
    }
    
    // Check if FTS has required columns
    let mut stmt = conn.prepare("PRAGMA table_info(message_fts)").map_err(|e| format!("Failed to query message_fts schema: {}", e))?;
    let fts_columns: Vec<String> = stmt.query_map([], |row| row.get(1))
        .map_err(|e| format!("Failed to read message_fts columns: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect message_fts columns: {}", e))?;
    let required_fts_columns = ["session_path", "role", "content"];
    let fts_has_all = required_fts_columns.iter().all(|&col| fts_columns.contains(&col.to_string()));
    
    if !fts_has_all {
        error!("[Migration] message_fts schema incomplete. Has columns: {:?}. Recreating virtual table...", fts_columns);
        conn.execute("DROP TABLE IF EXISTS message_fts", [])
            .map_err(|e| format!("Failed to drop old message_fts: {}", e))?;
        create_message_fts5(conn)?;
        conn.execute("INSERT INTO message_fts(message_fts) VALUES('rebuild')", [])
            .map_err(|e| format!("Failed to rebuild message_fts after schema fix: {}", e))?;
        info!("[Migration] Recreated message_fts virtual table and rebuilt index");
    } else {
        debug!("[Schema] message_fts columns OK: {:?}", fts_columns);
        // No triggers needed for content-bearing FTS5; automatic sync
    }
    
    Ok(())
}

// Ensure the triggers on message_entries exist and are up-to-date


fn create_message_fts5(conn: &Connection) -> Result<(), String> {
    // Create virtual FTS5 table that is automatically maintained by SQLite
    // because it specifies content='message_entries'. No triggers needed.
    conn.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS message_fts USING fts5(
            session_path UNINDEXED,
            role,
            content,
            content='message_entries',
            content_rowid='rowid',
            tokenize='unicode61'
        )",
        [],
    ).map_err(|e| format!("Failed to create message_fts: {}", e))?;

    info!("[FTS] Created message_fts virtual table (content auto-sync enabled)");
    Ok(())
}

pub fn upsert_session(conn: &Connection, session: &SessionInfo, file_modified: DateTime<Utc>) -> Result<(), String> {
    conn.execute(
        "INSERT INTO sessions (id, path, cwd, name, created, modified, file_modified, message_count, first_message, all_messages_text, user_messages_text, assistant_messages_text, last_message, last_message_role, cached_at, access_count, last_accessed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, 0, NULL)
         ON CONFLICT(path) DO UPDATE SET
            modified = excluded.modified,
            file_modified = excluded.file_modified,
            message_count = excluded.message_count,
            first_message = excluded.first_message,
            all_messages_text = excluded.all_messages_text,
            user_messages_text = excluded.user_messages_text,
            assistant_messages_text = excluded.assistant_messages_text,
            last_message = excluded.last_message,
            last_message_role = excluded.last_message_role,
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
            &session.user_messages_text,
            &session.assistant_messages_text,
            &session.last_message,
            &session.last_message_role,
            &Utc::now().to_rfc3339(),
        ],
    ).map_err(|e| format!("Failed to upsert session: {}", e))?;

    // Populate message_entries table if it exists (for per-message FTS)
    if conn.query_row(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='message_entries'",
        [],
        |row| Ok(row.get::<_, String>(0)?)
    ).map(|_| true).unwrap_or(false) {
        debug!("[Upsert] Updating message entries for session: {}", session.path);
        // Clear existing entries for this session to avoid duplicates
        delete_message_entries_for_session(conn, &session.path)?;
        // Insert fresh entries
        insert_message_entries(conn, session)?;
        debug!("[Upsert] Completed message entries for session: {}", session.path);
    } else {
        debug!("[Upsert] message_entries table does not exist, skipping");
    }

    Ok(())
}

pub fn get_session(conn: &Connection, path: &str) -> Result<Option<SessionInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text, user_messages_text, assistant_messages_text, last_message, last_message_role
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
            user_messages_text: row.get(9)?,
            assistant_messages_text: row.get(10)?,
            last_message: row.get(11)?,
            last_message_role: row.get(12)?,
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
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text, user_messages_text, assistant_messages_text, last_message, last_message_role
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
            user_messages_text: row.get(9)?,
            assistant_messages_text: row.get(10)?,
            last_message: row.get(11)?,
            last_message_role: row.get(12)?,
        })
    }).map_err(|e| format!("Failed to query sessions: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect sessions: {}", e))?;

    Ok(sessions)
}

pub fn get_sessions_modified_after(conn: &Connection, cutoff: DateTime<Utc>) -> Result<Vec<SessionInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text, user_messages_text, assistant_messages_text, last_message, last_message_role
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
            user_messages_text: row.get(9)?,
            assistant_messages_text: row.get(10)?,
            last_message: row.get(11)?,
            last_message_role: row.get(12)?,
        })
    }).map_err(|e| format!("Failed to query sessions: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect sessions: {}", e))?;

    Ok(sessions)
}

pub fn get_sessions_modified_before(conn: &Connection, cutoff: DateTime<Utc>) -> Result<Vec<SessionInfo>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text, user_messages_text, assistant_messages_text, last_message, last_message_role
         FROM sessions WHERE modified <= ? ORDER BY modified DESC"
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
            user_messages_text: row.get(9)?,
            assistant_messages_text: row.get(10)?,
            last_message: row.get(11)?,
            last_message_role: row.get(12)?,
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

pub fn needs_reindexing(conn: &Connection, path: &str) -> Result<bool, String> {
    let mut stmt = conn.prepare(
        "SELECT user_messages_text IS NULL OR assistant_messages_text IS NULL FROM sessions WHERE path = ?"
    ).map_err(|e| format!("Failed to prepare statement: {}", e))?;

    let result = stmt.query_row(params![path], |row| {
        Ok(row.get::<_, bool>(0)?)
    }).optional().map_err(|e| format!("Failed to query needs_reindexing: {}", e))?;

    Ok(result.unwrap_or(false))
}

pub fn delete_session(conn: &Connection, path: &str) -> Result<(), String> {
    // Also delete from message_entries (FOREIGN KEY CASCADE should handle but we do explicitly for safety)
    let _ = delete_message_entries_for_session(conn, path);
    
    conn.execute("DELETE FROM sessions WHERE path = ?", params![path])
        .map_err(|e| format!("Failed to delete session: {}", e))?;
    Ok(())
}

pub fn get_session_count(conn: &Connection) -> Result<usize, String> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .map_err(|e| format!("Failed to count sessions: {}", e))?;
    Ok(count as usize)
}

#[derive(Debug, Clone)]
pub struct SessionDetailsCache {
    pub file_modified: DateTime<Utc>,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub cache_read_tokens: usize,
    pub cache_write_tokens: usize,
    pub input_cost: f64,
    pub output_cost: f64,
    pub cache_read_cost: f64,
    pub cache_write_cost: f64,
    pub models_json: String,
}

pub fn get_session_details_cache(conn: &Connection, path: &str) -> Result<Option<SessionDetailsCache>, String> {
    let mut stmt = conn.prepare(
        "SELECT file_modified, user_messages, assistant_messages, input_tokens, output_tokens,
                cache_read_tokens, cache_write_tokens, input_cost, output_cost, cache_read_cost,
                cache_write_cost, models_json
         FROM session_details_cache
         WHERE path = ?"
    ).map_err(|e| format!("Failed to prepare session_details_cache statement: {}", e))?;

    let row = stmt.query_row(params![path], |row| {
        Ok(SessionDetailsCache {
            file_modified: parse_timestamp(&row.get::<_, String>(0)?),
            user_messages: row.get::<_, i64>(1)? as usize,
            assistant_messages: row.get::<_, i64>(2)? as usize,
            input_tokens: row.get::<_, i64>(3)? as usize,
            output_tokens: row.get::<_, i64>(4)? as usize,
            cache_read_tokens: row.get::<_, i64>(5)? as usize,
            cache_write_tokens: row.get::<_, i64>(6)? as usize,
            input_cost: row.get::<_, f64>(7)?,
            output_cost: row.get::<_, f64>(8)?,
            cache_read_cost: row.get::<_, f64>(9)?,
            cache_write_cost: row.get::<_, f64>(10)?,
            models_json: row.get::<_, String>(11)?,
        })
    }).ok();

    Ok(row)
}

pub fn upsert_session_details_cache(
    conn: &Connection,
    path: &str,
    file_modified: DateTime<Utc>,
    details: &SessionDetails,
) -> Result<(), String> {
    let models_json = serde_json::to_string(&details.models)
        .map_err(|e| format!("Failed to serialize models: {}", e))?;

    conn.execute(
        "INSERT INTO session_details_cache (
            path, file_modified, user_messages, assistant_messages, input_tokens, output_tokens,
            cache_read_tokens, cache_write_tokens, input_cost, output_cost, cache_read_cost,
            cache_write_cost, models_json
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        ON CONFLICT(path) DO UPDATE SET
            file_modified = excluded.file_modified,
            user_messages = excluded.user_messages,
            assistant_messages = excluded.assistant_messages,
            input_tokens = excluded.input_tokens,
            output_tokens = excluded.output_tokens,
            cache_read_tokens = excluded.cache_read_tokens,
            cache_write_tokens = excluded.cache_write_tokens,
            input_cost = excluded.input_cost,
            output_cost = excluded.output_cost,
            cache_read_cost = excluded.cache_read_cost,
            cache_write_cost = excluded.cache_write_cost,
            models_json = excluded.models_json",
        params![
            path,
            &file_modified.to_rfc3339(),
            details.user_messages as i64,
            details.assistant_messages as i64,
            details.input_tokens as i64,
            details.output_tokens as i64,
            details.cache_read_tokens as i64,
            details.cache_write_tokens as i64,
            details.input_cost,
            details.output_cost,
            details.cache_read_cost,
            details.cache_write_cost,
            models_json,
        ],
    ).map_err(|e| format!("Failed to upsert session_details_cache: {}", e))?;

    Ok(())
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
        "SELECT id, path, cwd, name, created, modified, message_count, first_message, all_messages_text, user_messages_text, assistant_messages_text, last_message, last_message_role
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
            user_messages_text: row.get(9)?,
            assistant_messages_text: row.get(10)?,
            last_message: row.get(11)?,
            last_message_role: row.get(12)?,
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

// ============ Message-level FTS support ============

/// Delete all message entries for a session (used before re-inserting)
pub fn delete_message_entries_for_session(conn: &Connection, session_path: &str) -> Result<(), String> {
    debug!("[Delete] Attempting to delete message entries for session: {}", session_path);
    
    // Check if message_entries table exists
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='message_entries'")
        .map_err(|e| format!("Failed to check message_entries existence: {}", e))?;
    let exists: bool = stmt.query_row([], |row| Ok(row.get::<_, String>(0)? == "message_entries"))
        .unwrap_or(false);
    
    if !exists {
        debug!("[Delete] message_entries table does not exist, skipping delete");
        return Ok(());
    }
    
    // Validate schema has all required columns before attempting DELETE
    let mut col_stmt = conn.prepare("PRAGMA table_info(message_entries)").map_err(|e| format!("Failed to query message_entries schema: {}", e))?;
    let column_names: Vec<String> = col_stmt.query_map([], |row| row.get(1))
        .map_err(|e| format!("Failed to read message_entries column names: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect message_entries column names: {}", e))?;
    
    let required = ["id", "session_path", "role", "content", "timestamp"];
    let has_all = required.iter().all(|&col| column_names.contains(&col.to_string()));
    
    if !has_all {
        error!("[Delete] message_entries schema incomplete. Columns: {:?}. Required: {:?}. Triggering migration...", 
            column_names, required);
        crate::sqlite_cache::ensure_message_fts_schema(conn)?;
        // Retry after migration
        let mut col_stmt2 = conn.prepare("PRAGMA table_info(message_entries)")
            .map_err(|e| format!("Failed to prepare PRAGMA after migration: {}", e))?;
        let columns2: Vec<String> = col_stmt2.query_map([], |row| row.get(1))
            .map_err(|e| format!("Failed to query columns after migration: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect columns after migration: {}", e))?;
        if !required.iter().all(|&col| columns2.contains(&col.to_string())) {
            return Err(format!("message_entries schema still incomplete after migration: {:?}", columns2));
        }
        debug!("[Delete] Schema migration successful, retrying delete");
        return delete_message_entries_for_session(conn, session_path);
    }
    
    // Debug logging
    if cfg!(debug_assertions) {
        debug!("[Delete] message_entries schema OK: {:?}", column_names);
    }
    
    match conn.execute("DELETE FROM message_entries WHERE session_path = ?", params![session_path]) {
        Ok(_) => {
            debug!("[Delete] Deleted message entries for session: {}", session_path);
        }
        Err(e) => {
            let err_str = format!("{:?}", e);
            error!("[Delete] Failed to delete message entries for session '{}': {:?} (code: {:?})", 
                session_path, e, e.sqlite_error_code());
            
            // Always attempt recovery via migration, then retry
            error!("[Delete] Attempting schema migration recovery...");
            if let Err(migrate_err) = crate::sqlite_cache::ensure_message_fts_schema(conn) {
                error!("[Delete] Migration recovery failed: {}", migrate_err);
            } else if let Ok(count) = conn.execute("DELETE FROM message_entries WHERE session_path = ?", params![session_path]) {
                debug!("[Delete] Recovered: deleted {} rows", count);
                return Ok(());
            }
            
            return Err(format!("Failed to delete message entries: {}", err_str));
        }
    }
    Ok(())
}

/// Insert message entries from a session file into message_entries table
pub fn insert_message_entries(conn: &Connection, session: &SessionInfo) -> Result<(), String> {
    use serde_json::Value;
    use std::io::BufReader;
    
    // Check if message_entries table exists (FTS may be disabled)
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='message_entries'")
        .map_err(|e| format!("Failed to check message_entries existence: {}", e))?;
    let exists: bool = stmt.query_row([], |row| Ok(row.get::<_, String>(0)? == "message_entries"))
        .unwrap_or(false);
    
    if !exists {
        // FTS is disabled or table not created yet - skip
        return Ok(());
    }
    
    let file = fs::File::open(&session.path)
        .map_err(|e| format!("Failed to open file for message entries: {}", e))?;
    let reader = BufReader::new(file);
    
    let mut inserted_count = 0;
    for line_result in reader.lines() {
        let line: String = line_result.map_err(|e| e.to_string())?;
        if line.trim().is_empty() { continue; }
        
        if let Ok(entry) = serde_json::from_str::<Value>(&line) {
            if entry["type"] == "message" {
                if let Some(message) = entry.get("message") {
                    let role = message["role"].as_str().unwrap_or("");
                    if role == "user" || role == "assistant" {
                        let entry_id = entry["id"].as_str().unwrap_or("").to_string();
                        if entry_id.is_empty() { continue; }
                        
                        let timestamp = entry["timestamp"].as_str().unwrap_or("").to_string();
                        
                        // Extract text content (exclude thinking)
                        let mut content = String::new();
                        if let Some(content_arr) = message["content"].as_array() {
                            for item in content_arr {
                                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                    content.push_str(text);
                                }
                            }
                        }
                        
                        if !content.is_empty() {
                            conn.execute(
                                "INSERT OR REPLACE INTO message_entries (id, session_path, role, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
                                params![
                                    &entry_id,
                                    &session.path,
                                    role,
                                    &content,
                                    &timestamp,
                                ],
                            ).map_err(|e| format!("Failed to insert message entry (session: {}, entry: {}): {}", 
                                session.path, entry_id, e))?;
                            inserted_count += 1;
                        }
                    }
                }
            }
        }
    }
    
    debug!("Inserted {} message entries for session: {}", inserted_count, session.path);
    Ok(())
}

/// Search message-level FTS5 index and return matching message entries
/// Returns (entry_id, session_path, role, snippet, timestamp, rank)
pub fn search_message_fts(
    conn: &Connection,
    query: &str,
    role_filter: Option<&str>,
    limit: usize
) -> Result<Vec<(String, String, String, String, String, f32)>, String> {
    // Build FTS query with optional role filter
    let clean_query = query.trim().replace("'", "''");
    if clean_query.is_empty() {
        return Ok(vec![]);
    }
    
    // Format query for FTS5: use prefix matching with *
    let query_terms: Vec<&str> = clean_query.split_whitespace().collect();
    let formatted_terms = query_terms.iter()
        .map(|term| format!("\"{}\"*", term))
        .collect::<Vec<String>>()
        .join(" OR ");
    
    // Build column filter
    let (fts_column, role_condition) = match role_filter {
        Some("user") => ("content", "m.role = 'user'"),
        Some("assistant") => ("content", "m.role = 'assistant'"),
        _ => ("content", "1=1"),
    };
    
    let fts_query = format!("{}: ({})", fts_column, formatted_terms);
    
    let sql = format!(
        "SELECT \
            m.id, \
            m.session_path, \
            m.role, \
            snippet(message_fts, 2, '<b>', '</b>', '...', 80) as snippet, \
            m.timestamp, \
            f.rank \
         FROM message_entries m \
         JOIN message_fts f ON m.rowid = f.rowid \
         WHERE f.message_fts MATCH ? \
         AND {} \
         ORDER BY f.rank \
         LIMIT ?",
        role_condition
    );
    
    let mut stmt = conn.prepare(&sql)
        .map_err(|e| format!("Failed to prepare message FTS query: {}", e))?;
    
    let rows = stmt.query_map(params![fts_query, limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,   // entry_id
            row.get::<_, String>(1)?,   // session_path
            row.get::<_, String>(2)?,   // role
            row.get::<_, String>(3)?,   // snippet with <b> tags
            row.get::<_, String>(4)?,   // timestamp
            row.get::<_, f32>(5)?,      // rank
        ))
    })
    .map_err(|e| format!("Failed to query message FTS: {}", e))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("Failed to collect message FTS results: {}", e))?;
    
    Ok(rows)
}

/// Rebuild message FTS index for all existing sessions
pub fn rebuild_message_fts(conn: &Connection) -> Result<(), String> {
    // Clear existing message entries and rebuild from all sessions
    conn.execute("DELETE FROM message_entries", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM message_fts", []).map_err(|e| e.to_string())?;
    
    // Get all session paths
    let mut stmt = conn.prepare("SELECT path FROM sessions")
        .map_err(|e| e.to_string())?;
    let paths: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    
    // For each session, parse and insert message entries
    for path in paths {
        if let Ok(Some(session)) = get_session(conn, &path) {
            insert_message_entries(conn, &session)?;
        }
    }
    
    Ok(())
}

fn parse_timestamp(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}

// Favorites functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbFavoriteItem {
    pub id: String,
    #[serde(rename = "type")]
    pub favorite_type: String,
    pub name: String,
    pub path: String,
    pub added_at: String,
}

pub fn add_favorite(conn: &Connection, id: &str, favorite_type: &str, name: &str, path: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO favorites (id, type, name, path, added_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, favorite_type, name, path, Utc::now().to_rfc3339()],
    ).map_err(|e| format!("Failed to add favorite: {}", e))?;
    Ok(())
}

pub fn remove_favorite(conn: &Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM favorites WHERE id = ?", params![id])
        .map_err(|e| format!("Failed to remove favorite: {}", e))?;
    Ok(())
}

pub fn get_all_favorites(conn: &Connection) -> Result<Vec<DbFavoriteItem>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, type, name, path, added_at FROM favorites ORDER BY added_at DESC"
    ).map_err(|e| format!("Failed to prepare favorites statement: {}", e))?;

    let favorites = stmt.query_map([], |row| {
        Ok(DbFavoriteItem {
            id: row.get(0)?,
            favorite_type: row.get(1)?,
            name: row.get(2)?,
            path: row.get(3)?,
            added_at: row.get(4)?,
        })
    }).map_err(|e| format!("Failed to query favorites: {}", e))?
        .collect::<SqliteResult<Vec<_>>>()
        .map_err(|e| format!("Failed to collect favorites: {}", e))?;

    Ok(favorites)
}

pub fn is_favorite(conn: &Connection, id: &str) -> Result<bool, String> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM favorites WHERE id = ?", params![id], |row| row.get(0))
        .map_err(|e| format!("Failed to check favorite: {}", e))?;
    Ok(count > 0)
}

pub fn toggle_favorite(conn: &Connection, id: &str, favorite_type: &str, name: &str, path: &str) -> Result<bool, String> {
    let exists = is_favorite(conn, id)?;
    if exists {
        remove_favorite(conn, id)?;
        Ok(false)
    } else {
        add_favorite(conn, id, favorite_type, name, path)?;
        Ok(true)
    }
}

pub fn full_rebuild_fts(conn: &Connection) -> Result<(), String> {
    conn.execute("DROP TABLE IF EXISTS sessions_fts", []).map_err(|e| e.to_string())?;
    conn.execute("DROP TRIGGER IF EXISTS sessions_ai", []).map_err(|e| e.to_string())?;
    conn.execute("DROP TRIGGER IF EXISTS sessions_ad", []).map_err(|e| e.to_string())?;
    conn.execute("DROP TRIGGER IF EXISTS sessions_au", []).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE VIRTUAL TABLE sessions_fts USING fts5(
            path UNINDEXED,
            cwd,
            name,
            first_message,
            all_messages_text,
            user_messages_text,
            assistant_messages_text,
            content='sessions',
            content_rowid='rowid',
            tokenize='unicode61'
        )",
        [],
    ).map_err(|e| format!("Failed to create FTS5 table: {}", e))?;

    conn.execute(
        "CREATE TRIGGER sessions_ai AFTER INSERT ON sessions BEGIN
            INSERT INTO sessions_fts(rowid, path, cwd, name, first_message, all_messages_text, user_messages_text, assistant_messages_text)
            VALUES (NEW.rowid, NEW.path, NEW.cwd, NEW.name, NEW.first_message, NEW.all_messages_text, NEW.user_messages_text, NEW.assistant_messages_text);
        END",
        [],
    ).map_err(|e| format!("Failed to create FTS5 insert trigger: {}", e))?;

    conn.execute(
        "CREATE TRIGGER sessions_ad AFTER DELETE ON sessions BEGIN
            INSERT INTO sessions_fts(sessions_fts, rowid, path, cwd, name, first_message, all_messages_text, user_messages_text, assistant_messages_text)
            VALUES ('delete', OLD.rowid, OLD.path, OLD.cwd, OLD.name, OLD.first_message, OLD.all_messages_text, OLD.user_messages_text, OLD.assistant_messages_text);
        END",
        [],
    ).map_err(|e| format!("Failed to create FTS5 delete trigger: {}", e))?;

    conn.execute(
        "CREATE TRIGGER sessions_au AFTER UPDATE ON sessions BEGIN
            INSERT INTO sessions_fts(sessions_fts, rowid, path, cwd, name, first_message, all_messages_text, user_messages_text, assistant_messages_text)
            VALUES ('delete', OLD.rowid, OLD.path, OLD.cwd, OLD.name, OLD.first_message, OLD.all_messages_text, OLD.user_messages_text, OLD.assistant_messages_text);
            INSERT INTO sessions_fts(rowid, path, cwd, name, first_message, all_messages_text, user_messages_text, assistant_messages_text)
            VALUES (NEW.rowid, NEW.path, NEW.cwd, NEW.name, NEW.first_message, NEW.all_messages_text, NEW.user_messages_text, NEW.assistant_messages_text);
        END",
        [],
    ).map_err(|e| format!("Failed to create FTS5 update trigger: {}", e))?;

    // Rebuild the index from existing sessions
    conn.execute("INSERT INTO sessions_fts(sessions_fts) VALUES('rebuild')", []).map_err(|e| e.to_string())?;

    Ok(())
}
