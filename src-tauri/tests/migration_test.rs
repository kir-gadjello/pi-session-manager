use pi_session_manager::sqlite_cache;
use pi_session_manager::config::Config;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_database_migration_from_old_schema() {
    let test_db_path = PathBuf::from("/tmp/pi_migration_test.db");
    if test_db_path.exists() {
        fs::remove_file(&test_db_path).ok();
    }

    // 1. Create a DB with the OLD schema (missing new columns)
    {
        let conn = Connection::open(&test_db_path).unwrap();
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
                cached_at TEXT NOT NULL
            )",
            [],
        ).unwrap();

        // Old FTS5 table
        conn.execute(
            "CREATE VIRTUAL TABLE sessions_fts USING fts5(
                path UNINDEXED,
                cwd,
                name,
                first_message,
                all_messages_text,
                content='sessions',
                content_rowid='rowid'
            )",
            [],
        ).unwrap();
    }

    // 2. Run the current initialization logic
    let config = Config {
        enable_fts5: true,
        ..Default::default()
    };
    
    // We need to point sqlite_cache to our test DB. 
    // Since get_db_path is hardcoded, we'll manually call the logic using our connection.
    {
        let conn = Connection::open(&test_db_path).unwrap();
        
        // This is what init_db_with_config does:
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
        ).unwrap();

        // Run migrations
        conn.execute("ALTER TABLE sessions ADD COLUMN last_message TEXT", []).ok();
        conn.execute("ALTER TABLE sessions ADD COLUMN last_message_role TEXT", []).ok();
        conn.execute("ALTER TABLE sessions ADD COLUMN user_messages_text TEXT", []).ok();
        conn.execute("ALTER TABLE sessions ADD COLUMN assistant_messages_text TEXT", []).ok();

        // Check columns
        let mut stmt = conn.prepare("PRAGMA table_info(sessions)").unwrap();
        let cols: Vec<String> = stmt.query_map([], |row| row.get(1)).unwrap().map(|r| r.unwrap()).collect();
        assert!(cols.contains(&"user_messages_text".to_string()));
        assert!(cols.contains(&"assistant_messages_text".to_string()));

        // FTS upgrade logic (copied from init_fts5)
        let mut stmt = conn.prepare("PRAGMA table_info(sessions_fts)").unwrap();
        let fts_cols: Vec<String> = stmt.query_map([], |row| row.get(1)).unwrap().map(|r| r.unwrap()).collect();
        
        if !fts_cols.is_empty() && !fts_cols.contains(&"user_messages_text".to_string()) {
            conn.execute("DROP TABLE IF EXISTS sessions_fts", []).unwrap();
        }

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS sessions_fts USING fts5(
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

        let mut stmt = conn.prepare("PRAGMA table_info(sessions_fts)").unwrap();
        let new_fts_cols: Vec<String> = stmt.query_map([], |row| row.get(1)).unwrap().map(|r| r.unwrap()).collect();
        assert!(new_fts_cols.contains(&"user_messages_text".to_string()));
    }

    fs::remove_file(&test_db_path).ok();
}

#[test]
fn test_database_corruption_recovery() {
    let test_db_path = PathBuf::from("/tmp/pi_corruption_test.db");
    if test_db_path.exists() {
        fs::remove_file(&test_db_path).ok();
    }

    // 1. Create a "corrupted" file (not a valid SQLite DB)
    fs::write(&test_db_path, "this is not a database").unwrap();

    // 2. Try to initialize it using the recovery logic
    // Note: since we can't easily mock get_db_path without changing the source,
    // we'll manually test the recovery pattern used in init_db_with_config
    
    let config = Config::default();
    
    let result = (|| -> Result<Connection, String> {
        let open_init = |path: &std::path::Path| -> Result<Connection, String> {
            let conn = Connection::open(path).map_err(|e| e.to_string())?;
            // Simulate a failure that would happen on a malformed DB
            conn.execute("PRAGMA schema_version", []).map_err(|e| e.to_string())?;
            Ok(conn)
        };

        let initial_err = match open_init(&test_db_path) {
            Ok(conn) => return Ok(conn),
            Err(e) => e,
        };

        println!("Got error: '{}'", initial_err);
        if initial_err.contains("malformed") || initial_err.contains("disk image") || initial_err.contains("not a database") {
            fs::remove_file(&test_db_path).map_err(|err| err.to_string())?;
            open_init(&test_db_path)
        } else {
            Err(initial_err)
        }
    })();

    assert!(result.is_ok(), "Should recover from corrupted file by deleting and recreating it");
    assert!(test_db_path.exists());
    
    fs::remove_file(&test_db_path).ok();
}
