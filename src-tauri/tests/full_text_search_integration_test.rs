use chrono::Utc;
use lazy_static::lazy_static;
use pi_session_manager::commands::full_text_search;
use pi_session_manager::config::Config;
use pi_session_manager::models::{FullTextSearchHit, FullTextSearchResponse};
use pi_session_manager::scanner;
use pi_session_manager::sqlite_cache;
use rusqlite::{params, Connection};
use std::env;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

lazy_static! {
    static ref TEST_DB_LOCK: Mutex<()> = Mutex::new(());
}

/// Helper: create a minimal session file with multiple messages
fn make_session_file(id: &str, cwd: &str, messages: &[(&str, &str)]) -> String {
    let header = format!(
        r#"{{"type":"session","version":3,"id":"{}","timestamp":"2026-02-10T22:00:00Z","cwd":"{}"}}"#,
        id, cwd
    );
    let mut lines = vec![header];
    for (i, (role, text)) in messages.iter().enumerate() {
        let entry_id = format!("{}-msg{}", id, i);
        let timestamp = format!("2026-02-10T22:00:{:02}Z", i);
        // Escape backslashes and double quotes for JSON
        let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
        let msg = format!(
            r#"{{"type":"message","id":"{}","parentId":null,"timestamp":"{}","message":{{"role":"{}","content":[{{"type":"text","text":"{}"}}]}}}}"#,
            entry_id, timestamp, role, escaped
        );
        lines.push(msg);
    }
    lines.join("\n")
}

/// Set up a test database with sessions and message entries
fn setup_test_db(sessions: &[(&str, &str, &[(&str, &str)])]) -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let sessions_dir = temp_dir.path().join("sessions");
    fs::create_dir_all(&sessions_dir).unwrap();

    // Set HOME so config uses temp dir for DB
    let original_home = env::var("HOME").ok();
    env::set_var("HOME", temp_dir.path());

    let config = Config::default();
    let conn = sqlite_cache::init_db_with_config(&config).unwrap();

    for (id, cwd, messages) in sessions {
        let path = sessions_dir.join(format!("{}.jsonl", id));
        let content = make_session_file(id, cwd, messages);
        fs::write(&path, content).unwrap();

        let (session, entries) = scanner::parse_session_info(&path).unwrap();
        sqlite_cache::upsert_session(&conn, &session, Utc::now()).unwrap();
        sqlite_cache::upsert_message_entries(&conn, &session.path, &entries).unwrap();
    }

    // Drop connection to avoid locks when full_text_search opens its own connection
    drop(conn);

    temp_dir
}

#[tokio::test]
async fn test_full_text_search_command_basic() {
    // Acquire global test lock to prevent concurrent DB access
    let _lock = TEST_DB_LOCK.lock().unwrap();

    // Setup sessions
    let _temp_dir = setup_test_db(&[
        (
            "sess1",
            "/cwd1",
            &[
                ("user", "I like banana and apple"),
                ("assistant", "Here is a banana recipe"),
                ("user", "banana smoothie recipe"),
            ],
        ),
        (
            "sess2",
            "/cwd2",
            &[
                ("user", "How to learn Rust?"),
                (
                    "assistant",
                    "Rust is a systems programming language with ownership",
                ),
                ("user", "Is Rust safe?"),
                (
                    "assistant",
                    "Yes, Rust guarantees memory safety without garbage collection",
                ),
            ],
        ),
        (
            "sess3",
            "/cwd3",
            &[
                ("user", "How to use tokio?"),
                (
                    "assistant",
                    "Tokio is an async runtime for Rust. Use tokio::main.",
                ),
            ],
        ),
    ]);

    // Test 1: Search for "banana"
    let response: FullTextSearchResponse =
        full_text_search("banana".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();

    assert!(
        response.total_hits >= 1,
        "Expected at least 1 hit for 'banana'"
    );
    assert!(
        response.hits.iter().any(|h| h.session_id == "sess1"),
        "Expected sess1 to be in results"
    );
    assert!(response.hits.iter().all(|h| h.session_id == "sess1"));
    assert!(response.hits[0].score > 0.0);

    // Test 2: Search for "rust"
    let response: FullTextSearchResponse =
        full_text_search("rust".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();

    assert!(response.total_hits >= 2);
    let sess_ids: Vec<String> = response.hits.iter().map(|h| h.session_id.clone()).collect();
    assert!(sess_ids.contains(&"sess2".to_string()));
    assert!(sess_ids.contains(&"sess3".to_string()));

    // Test 3: Role filter - user only on "banana"
    let response: FullTextSearchResponse =
        full_text_search("banana".to_string(), "user".to_string(), None, 0, 10)
            .await
            .unwrap();

    assert!(!response.hits.is_empty());
    assert!(response.hits.iter().all(|h| h.role == "user"));
    assert!(!response.hits.iter().any(|h| h.role == "assistant"));

    // Test 4: Role filter - assistant only on "banana"
    let response: FullTextSearchResponse =
        full_text_search("banana".to_string(), "assistant".to_string(), None, 0, 10)
            .await
            .unwrap();

    assert!(!response.hits.is_empty());
    assert!(response.hits.iter().all(|h| h.role == "assistant"));

    // Test 5: Empty query
    let response: FullTextSearchResponse =
        full_text_search("".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert_eq!(response.total_hits, 0);
    assert!(response.hits.is_empty());

    // Test 6: No match
    let response: FullTextSearchResponse =
        full_text_search("xyznonexistent".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert_eq!(response.total_hits, 0);
    assert!(response.hits.is_empty());

    // Test 7: Pagination
    let page0: FullTextSearchResponse =
        full_text_search("rust".to_string(), "all".to_string(), None, 0, 2)
            .await
            .unwrap();
    assert!(page0.total_hits >= 2);
    assert!(page0.hits.len() <= 2);

    let page1: FullTextSearchResponse =
        full_text_search("rust".to_string(), "all".to_string(), None, 1, 2)
            .await
            .unwrap();
    let total_from_pages = page0.hits.len() + page1.hits.len();
    assert!(total_from_pages <= page0.total_hits);

    // Test 8: Glob pattern
    let response: FullTextSearchResponse = full_text_search(
        "banana".to_string(),
        "all".to_string(),
        Some("/cwd1/*".to_string()),
        0,
        10,
    )
    .await
    .unwrap();
    assert!(response
        .hits
        .iter()
        .all(|h| h.session_path.contains("/cwd1")));

    // Test 9: Score is positive
    let response: FullTextSearchResponse =
        full_text_search("banana".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    for hit in &response.hits {
        assert!(hit.score > 0.0);
    }

    println!("✅ All full_text_search command tests passed!");
}

#[tokio::test]
async fn test_full_text_search_pagination_across_sessions() {
    let _lock = TEST_DB_LOCK.lock().unwrap();

    let _temp_dir = setup_test_db(&[
        ("s1", "/cwd1", &[("user", "apple"); 5]),
        ("s2", "/cwd2", &[("user", "banana"); 5]),
        ("s3", "/cwd3", &[("user", "cherry"); 5]),
    ]);

    // Query "banana" with page size 3, per-session limit 3
    let page0: FullTextSearchResponse =
        full_text_search("banana".to_string(), "all".to_string(), None, 0, 3)
            .await
            .unwrap();

    // sess2 has 5 matches but per-session limit is 3
    assert_eq!(page0.total_hits, 3);
    assert_eq!(page0.hits.len(), 3);
    assert!(page0.hits.iter().all(|h| h.session_id == "s2"));

    // Query "apple" with page size 10
    let page0: FullTextSearchResponse =
        full_text_search("apple".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert_eq!(page0.total_hits, 3);
    assert!(page0.hits.iter().all(|h| h.session_id == "s1"));

    println!("✅ Pagination across sessions test passed!");
}

#[tokio::test]
async fn test_full_text_search_result_structure() {
    let _lock = TEST_DB_LOCK.lock().unwrap();

    let _temp_dir = setup_test_db(&[(
        "s1",
        "/projects/test",
        &[("user", "Hello world"), ("assistant", "Hi there!")],
    )]);

    let response: FullTextSearchResponse =
        full_text_search("hello".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();

    assert!(!response.hits.is_empty());
    let hit = &response.hits[0];

    assert!(!hit.session_id.is_empty());
    assert!(!hit.session_path.is_empty());
    assert!(!hit.entry_id.is_empty());
    assert!(!hit.role.is_empty());
    assert!(!hit.content.is_empty());
    // Timestamp is DateTime<Utc>; convert to RFC3339 string
    let ts_str = hit.timestamp.to_rfc3339();
    assert!(!ts_str.is_empty());
    assert!(hit.score > 0.0);

    assert!(hit.role == "user" || hit.role == "assistant");
    assert!(hit.session_path.ends_with(".jsonl"));

    let dt = chrono::DateTime::parse_from_rfc3339(&ts_str);
    assert!(dt.is_ok());

    println!("✅ Result structure test passed!");
}

#[tokio::test]
async fn test_full_text_search_escaping_special_chars() {
    let _lock = TEST_DB_LOCK.lock().unwrap();

    let _temp_dir = setup_test_db(&[(
        "s1",
        "/cwd",
        &[
            ("user", r#"This has "double quotes" and \ backslash"#),
            ("assistant", r#"Also contains 'single' quotes"#),
        ],
    )]);

    // Search for "double quotes"
    let response: FullTextSearchResponse = full_text_search(
        r#"double quotes"#.to_string(),
        "all".to_string(),
        None,
        0,
        10,
    )
    .await
    .unwrap();
    assert!(!response.hits.is_empty());

    // Search for backslash
    let response: FullTextSearchResponse =
        full_text_search("backslash".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert!(!response.hits.is_empty());

    // Search with quote in query
    let response: FullTextSearchResponse =
        full_text_search(r#""double""#.to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    // Should not panic

    println!("✅ Special character escaping test passed!");
}

#[tokio::test]
async fn test_full_text_search_after_session_update() {
    let _lock = TEST_DB_LOCK.lock().unwrap();

    let temp_dir = setup_test_db(&[(
        "s1",
        "/cwd",
        &[
            ("user", "Initial content"),
            ("assistant", "Initial response"),
        ],
    )]);

    let sess_path = temp_dir.path().join("sessions").join("s1.jsonl");

    // Verify initial search
    let resp1: FullTextSearchResponse =
        full_text_search("Initial".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert!(!resp1.hits.is_empty());

    // Append new message (ensure newline separation)
    let additional = r#"{"type":"message","id":"new-msg","parentId":null,"timestamp":"2026-02-10T23:00:00Z","message":{"role":"user","content":[{"type":"text","text":"Updated with new content"}]}}"#;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&sess_path)
        .unwrap();
    use std::io::Write;
    writeln!(file, "\n{}", additional).unwrap();
    file.sync_all().unwrap();

    // Rescan changed file
    let changed_paths = vec![sess_path.to_string_lossy().to_string()];
    let _diff = scanner::rescan_changed_files(changed_paths).await.unwrap();

    // Search for "Updated"
    let resp2: FullTextSearchResponse =
        full_text_search("Updated".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert!(!resp2.hits.is_empty());
    assert!(resp2.hits.iter().any(|h| h.session_id == "s1"));

    // Original "Initial" should still be there
    let resp3: FullTextSearchResponse =
        full_text_search("Initial".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert!(!resp3.hits.is_empty());

    println!("✅ Session update test passed!");
}

#[tokio::test]
async fn test_full_text_search_cascade_delete() {
    let _lock = TEST_DB_LOCK.lock().unwrap();

    let temp_dir = setup_test_db(&[
        ("s1", "/cwd1", &[("user", "deleteme")]),
        ("s2", "/cwd2", &[("user", "keepme")]),
    ]);

    let sess1_path = temp_dir.path().join("sessions").join("s1.jsonl");
    let sess2_path = temp_dir.path().join("sessions").join("s2.jsonl");

    // Open a connection to perform DELETE
    let config = Config::default();
    let conn = sqlite_cache::init_db_with_config(&config).unwrap();

    // Verify both searchable
    let resp_before: FullTextSearchResponse =
        full_text_search("deleteme".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert!(!resp_before.hits.is_empty());

    // Delete session 1
    conn.execute(
        "DELETE FROM sessions WHERE path = ?",
        params![sess1_path.to_string_lossy().to_string()],
    )
    .unwrap();

    // Check cascade
    let me_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM message_entries WHERE session_path = ?",
            params![sess1_path.to_string_lossy().to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(me_count, 0);

    let fts_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM message_fts WHERE session_path = ?",
            params![sess1_path.to_string_lossy().to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(fts_count, 0);

    // Search for "deleteme" should not find anything
    let resp_after: FullTextSearchResponse =
        full_text_search("deleteme".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert!(resp_after.hits.is_empty());

    // Search for "keepme" should still work
    let resp_keep: FullTextSearchResponse =
        full_text_search("keepme".to_string(), "all".to_string(), None, 0, 10)
            .await
            .unwrap();
    assert!(!resp_keep.hits.is_empty());

    println!("✅ Cascade delete test passed!");
}
