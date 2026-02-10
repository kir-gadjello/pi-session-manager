use pi_session_manager::models::{SessionInfo};
use pi_session_manager::sqlite_cache;
use chrono::{Utc, TimeZone};
use std::fs;
use std::path::PathBuf;

fn setup_test_db() -> (PathBuf, rusqlite::Connection) {
    let test_db_dir = PathBuf::from("/tmp/pi_fts_test_staff_audit");
    if test_db_dir.exists() {
        fs::remove_dir_all(&test_db_dir).ok();
    }
    fs::create_dir_all(&test_db_dir).expect("Failed to create test db dir");
    
    let conn = rusqlite::Connection::open_in_memory().expect("Failed to open in-memory DB");
    
    // Manually initialize schema since we are using in-memory
    conn.execute(
        "CREATE TABLE sessions (
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
    ).unwrap();

    // Init FTS5 with unicode61 tokenizer
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
    ).unwrap();

    conn.execute(
        "CREATE TRIGGER sessions_ai AFTER INSERT ON sessions BEGIN
            INSERT INTO sessions_fts(rowid, path, cwd, name, first_message, all_messages_text, user_messages_text, assistant_messages_text)
            VALUES (NEW.rowid, NEW.path, NEW.cwd, NEW.name, NEW.first_message, NEW.all_messages_text, NEW.user_messages_text, NEW.assistant_messages_text);
        END",
        [],
    ).unwrap();

    (test_db_dir, conn)
}

#[tokio::test]
async fn test_full_text_search_integration() {
    let (_test_dir, conn) = setup_test_db();
    
    let session_path = "/tmp/pi_fts_test_staff_audit/session1.jsonl";
    fs::create_dir_all("/tmp/pi_fts_test_staff_audit").unwrap();
    fs::write(session_path, r#"{"type":"session","id":"s1","cwd":"/test","timestamp":"2025-01-01T00:00:00Z"}
{"type":"message","id":"m1","timestamp":"2025-01-01T00:00:01Z","message":{"role":"user","content":[{"type":"text","text":"I love Rust programming"}]}}
{"type":"message","id":"m2","timestamp":"2025-01-01T00:00:02Z","message":{"role":"assistant","content":[{"type":"text","text":"Rust is indeed a great language"}]}}"#).unwrap();

    let session = SessionInfo {
        id: "s1".to_string(),
        path: session_path.to_string(),
        cwd: "/test".to_string(),
        name: Some("Rust Test".to_string()),
        created: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        modified: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
        message_count: 2,
        first_message: "I love Rust programming".to_string(),
        all_messages_text: "I love Rust programming
Rust is indeed a great language".to_string(),
        user_messages_text: "I love Rust programming".to_string(),
        assistant_messages_text: "Rust is indeed a great language".to_string(),
        last_message: "Rust is indeed a great language".to_string(),
        last_message_role: "assistant".to_string(),
    };

    sqlite_cache::upsert_session(&conn, &session, Utc::now()).unwrap();

    // Verify FTS5 is working (case-insensitive)
    let mut stmt = conn.prepare("SELECT path FROM sessions_fts WHERE sessions_fts MATCH 'rust'").unwrap();
    let paths: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().map(|r| r.unwrap()).collect();
    assert_eq!(paths.len(), 1, "Should match 'rust' case-insensitively");
    assert_eq!(paths[0], session_path);

    // Test role-specific FTS5
    let mut stmt = conn.prepare("SELECT path FROM sessions_fts WHERE user_messages_text MATCH 'Rust'").unwrap();
    let paths: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().map(|r| r.unwrap()).collect();
    assert_eq!(paths.len(), 1, "Should match in user messages");

    let mut stmt = conn.prepare("SELECT path FROM sessions_fts WHERE assistant_messages_text MATCH 'love'").unwrap();
    let paths: Vec<String> = stmt.query_map([], |row| row.get(0)).unwrap().map(|r| r.unwrap()).collect();
    assert_eq!(paths.len(), 0, "Should NOT match 'love' in assistant messages");

    fs::remove_dir_all("/tmp/pi_fts_test_staff_audit").ok();
}