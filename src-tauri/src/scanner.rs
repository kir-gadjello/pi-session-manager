use crate::config::Config;
use crate::models::SessionInfo;
use crate::sqlite_cache;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn get_sessions_dir() -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Cannot find home directory")?;
    Ok(home.join(".pi").join("agent").join("sessions"))
}

pub async fn scan_sessions() -> Result<Vec<SessionInfo>, String> {
    let config = Config::load().unwrap_or_default();
    scan_sessions_with_config(&config).await
}

pub async fn scan_sessions_with_config(config: &Config) -> Result<Vec<SessionInfo>, String> {
    let sessions_dir = get_sessions_dir()?;
    let realtime_cutoff = Utc::now() - Duration::days(config.realtime_cutoff_days);

    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut conn_opt = Some(sqlite_cache::init_db_with_config(config)?);
    let mut sessions: Vec<SessionInfo> = vec![];

    let entries = fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read sessions directory: {}", e))?;

    let process_entry = |conn: &rusqlite::Connection,
                         path: PathBuf|
     -> Result<Option<SessionInfo>, String> {
        if path.extension().map(|ext| ext == "jsonl").unwrap_or(false) {
            let path_str = path.to_string_lossy().to_string();
            let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
            let file_modified: DateTime<Utc> =
                DateTime::from(metadata.modified().unwrap_or(std::time::SystemTime::now()));

            if file_modified > realtime_cutoff {
                if let Ok(info) = parse_session_info(&path) {
                    sqlite_cache::upsert_session(conn, &info, file_modified)?;
                    return Ok(Some(info));
                }
            } else {
                let should_reparse = if let Some(cached_mtime) =
                    sqlite_cache::get_cached_file_modified(conn, &path_str)?
                {
                    file_modified > cached_mtime || sqlite_cache::needs_reindexing(conn, &path_str)?
                } else {
                    true
                };

                if should_reparse {
                    if let Ok(info) = parse_session_info(&path) {
                        sqlite_cache::upsert_session(conn, &info, file_modified)?;
                    }
                }
            }
        }
        Ok(None)
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(files) = fs::read_dir(&path) {
                for file in files.flatten() {
                    let file_path = file.path();

                    // Retry loop for corruption recovery
                    let mut retries = 0;
                    loop {
                        if retries > 1 {
                            eprintln!(
                                "Failed to process session {} after retry",
                                file_path.display()
                            );
                            break;
                        }

                        let conn = conn_opt.as_ref().ok_or("Database connection lost")?;
                        match process_entry(conn, file_path.clone()) {
                            Ok(Some(info)) => {
                                sessions.push(info);
                                break;
                            }
                            Ok(None) => break,
                            Err(e)
                                if e.contains("malformed")
                                    || e.contains("disk image")
                                    || e.contains("not a database") =>
                            {
                                eprintln!("[Scanner] Database corruption detected during scan: {}. Resetting...", e);
                                drop(conn_opt.take()); // Drop connection

                                // Get DB path to delete it
                                if let Ok(db_path) = sqlite_cache::get_db_path() {
                                    fs::remove_file(&db_path).ok();
                                }

                                // Re-init
                                conn_opt = Some(sqlite_cache::init_db_with_config(config)?);
                                retries += 1;
                            }
                            Err(e) => {
                                eprintln!(
                                    "Error processing session {}: {}",
                                    file_path.display(),
                                    e
                                );
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    // Historical sessions loading with retry
    let mut retries = 0;
    loop {
        if retries > 1 {
            break;
        }
        let conn = conn_opt.as_ref().ok_or("Database connection lost")?;

        match sqlite_cache::get_sessions_modified_before(conn, realtime_cutoff) {
            Ok(historical_sessions) => {
                for session in historical_sessions {
                    if !sessions.iter().any(|s| s.path == session.path) {
                        sessions.push(session);
                    }
                }
                break;
            }
            Err(e)
                if e.contains("malformed")
                    || e.contains("disk image")
                    || e.contains("not a database") =>
            {
                eprintln!(
                    "[Scanner] Database corruption loading history: {}. Resetting...",
                    e
                );
                drop(conn_opt.take());
                if let Ok(db_path) = sqlite_cache::get_db_path() {
                    fs::remove_file(&db_path).ok();
                }
                conn_opt = Some(sqlite_cache::init_db_with_config(config)?);
                retries += 1;
            }
            Err(e) => {
                eprintln!("Error loading historical sessions: {}", e);
                break;
            }
        }
    }

    // Stable sort: by modified descending, then by path ascending to guarantee deterministic order
    sessions.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.path.cmp(&b.path)));

    let realtime_count = sessions
        .iter()
        .filter(|s| s.modified > realtime_cutoff)
        .count();
    let historical_count = sessions.len() - realtime_count;

    eprintln!(
        "Scan complete: {} realtime (≤{}d), {} historical (>{}d), total {}",
        realtime_count,
        config.realtime_cutoff_days,
        historical_count,
        config.realtime_cutoff_days,
        sessions.len()
    );

    Ok(sessions)
}

/// 解析会话信息
/// 优化：使用 BufReader 流式读取，减少大文件内存占用
pub fn parse_session_info(path: &Path) -> Result<SessionInfo, String> {
    let file = fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // 读取并解析头部
    let header_line = lines
        .next()
        .ok_or("Empty session file")?
        .map_err(|e| format!("Failed to read header: {}", e))?;

    let header: Value =
        serde_json::from_str(&header_line).map_err(|e| format!("Failed to parse header: {}", e))?;

    if header["type"] != "session" {
        return Err("Invalid session header".to_string());
    }

    let id = header["id"].as_str().unwrap_or("unknown").to_string();
    let cwd = header["cwd"].as_str().unwrap_or("").to_string();
    let timestamp_str = header["timestamp"].as_str().unwrap_or("");
    let created = parse_timestamp(timestamp_str)?;

    let metadata = fs::metadata(path).map_err(|e| format!("Failed to get metadata: {}", e))?;
    let modified = DateTime::from(
        metadata
            .modified()
            .map_err(|e| format!("Failed to get modified time: {}", e))?,
    );

    let mut message_count = 0;
    let mut first_message = String::new();
    let mut all_messages = Vec::new();
    let mut user_messages = Vec::new();
    let mut assistant_messages = Vec::new();
    let mut name: Option<String> = None;
    let mut last_message = String::new();
    let mut last_message_role = String::new();

    // 流式读取剩余行，减少内存占用
    for line_result in lines {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<Value>(&line) {
            if entry["type"] == "session_info" {
                if let Some(n) = entry["name"].as_str() {
                    name = Some(n.trim().to_string());
                }
            }

            if entry["type"] == "message" {
                let role = entry["message"]["role"].as_str().unwrap_or("");
                if role == "user" || role == "assistant" {
                    message_count += 1;

                    let text = extract_message_text(&entry);
                    if !text.is_empty() {
                        all_messages.push(text.clone());
                        if role == "user" {
                            user_messages.push(text.clone());
                            if first_message.is_empty() {
                                first_message = text.chars().take(100).collect();
                            }
                        } else if role == "assistant" {
                            assistant_messages.push(text.clone());
                        }
                        // 更新最后一条消息
                        last_message = text.chars().take(150).collect();
                        last_message_role = role.to_string();
                    }
                }
            }
        }
    }

    let all_messages_text = all_messages.join("\n");
    let user_messages_text = user_messages.join("\n");
    let assistant_messages_text = assistant_messages.join("\n");

    Ok(SessionInfo {
        path: path.to_string_lossy().to_string(),
        id,
        cwd,
        name,
        created,
        modified,
        message_count,
        first_message,
        all_messages_text,
        user_messages_text,
        assistant_messages_text,
        last_message,
        last_message_role,
    })
}

fn extract_message_text(entry: &Value) -> String {
    if let Some(message) = entry.get("message") {
        if let Some(content) = message.get("content") {
            if content.is_array() {
                for item in content.as_array().unwrap() {
                    if let Some(text) = item.get("text") {
                        if let Some(s) = text.as_str() {
                            return s.to_string();
                        }
                    }
                }
            }
        }
    }
    String::new()
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Failed to parse timestamp: {}", e))
}
