use crate::models::{SessionInfo, FullTextSearchHit, FullTextSearchResponse};
use crate::{config, search, sqlite_cache};
use chrono::{DateTime, Utc};
use glob::Pattern;
use std::collections::HashMap;

#[tauri::command]
pub async fn search_sessions(
    sessions: Vec<SessionInfo>,
    query: String,
    search_mode: String,
    role_filter: String,
    include_tools: bool,
) -> Result<Vec<crate::models::SearchResult>, String> {
    let mode = match search_mode.as_str() {
        "name" => search::SearchMode::Name,
        _ => search::SearchMode::Content,
    };

    let role = match role_filter.as_str() {
        "user" => search::RoleFilter::User,
        "assistant" => search::RoleFilter::Assistant,
        _ => search::RoleFilter::All,
    };

    Ok(search::search_sessions(
        &sessions,
        &query,
        mode,
        role,
        include_tools,
    ))
}

#[tauri::command]
pub async fn search_sessions_fts(query: String, limit: usize) -> Result<Vec<SessionInfo>, String> {
    let config = config::load_config()?;
    let conn = sqlite_cache::init_db_with_config(&config)?;

    let paths = sqlite_cache::search_fts5(&conn, &query, limit)?;

    let mut sessions = Vec::new();
    for path in paths {
        if let Some(session) = sqlite_cache::get_session(&conn, &path)? {
            sessions.push(session);
        }
    }

    Ok(sessions)
}

#[tauri::command]
pub async fn full_text_search(
    query: String,
    role_filter: String,
    glob_pattern: Option<String>,
    page: usize,
    page_size: usize,
) -> Result<FullTextSearchResponse, String> {
    let config = config::load_config()?;
    let conn = sqlite_cache::init_db_with_config(&config)?;
    
    // Determine role filter for message FTS
    let role_opt = match role_filter.as_str() {
        "user" => Some("user"),
        "assistant" => Some("assistant"),
        _ => None,
    };
    
    // Fetch more results than needed to account for glob filtering and dedup
    let fetch_limit = (page + 1) * page_size + 100;
    
    // Use message-level FTS: returns (entry_id, session_path, role, snippet, timestamp, rank) directly from index
    let results = sqlite_cache::search_message_fts(&conn, &query, role_opt, fetch_limit)?;
    
    // Apply glob pattern and group by session
    let mut session_hits: HashMap<String, Vec<(String, String, String, String, f32)>> = 
        HashMap::new();
    
    for (entry_id, session_path, role, snippet_with_tags, timestamp_str, rank) in results {
        // Apply glob filter
        if let Some(pattern_str) = &glob_pattern {
            if !pattern_str.is_empty() {
                let pattern = Pattern::new(pattern_str)
                    .map_err(|e| format!("Invalid glob pattern: {}", e))?;
                if !pattern.matches(&session_path) {
                    continue;
                }
            }
        }
        
        // Remove HTML highlight tags
        let snippet = snippet_with_tags.replace("<b>", "").replace("</b>", "");
        
        session_hits.entry(session_path.clone())
            .or_default()
            .push((entry_id, role, snippet, timestamp_str, rank));
    }
    
    // Batch fetch session details and build hits
    let mut all_hits = Vec::new();
    let mut sessions_cache: HashMap<String, SessionInfo> = HashMap::new();
    
    for (session_path, mut entries) in session_hits {
        // Limit entries per session to distribute results fairly
        entries.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));
        let entries = entries.into_iter().take(3); // max 3 matches per session
        
        // Get session from cache or DB
        let session = if let Some(sess) = sessions_cache.get(&session_path) {
            sess.clone()
        } else if let Some(sess) = sqlite_cache::get_session(&conn, &session_path)? {
            sessions_cache.insert(session_path.clone(), sess.clone());
            sess
        } else {
            continue;
        };
        
        for (entry_id, role, snippet, timestamp_str, rank) in entries {
            // Timestamp is in storage, parse it
            // Note: timestamp from search_message_fts is the message timestamp string
            // We'll keep it as string to avoid parsing overhead, frontend can parse or display as needed
            // But FullTextSearchHit expects DateTime<Utc>. We can pass a placeholder or parse.
            // Let's parse it here:
            let timestamp = match chrono::DateTime::parse_from_rfc3339(&timestamp_str) {
                Ok(dt) => dt.with_timezone(&chrono::Utc),
                Err(_) => chrono::Utc::now(),
            };
            
            all_hits.push(FullTextSearchHit {
                session_id: session.id.clone(),
                session_path: session.path.clone(),
                session_name: session.name.clone(),
                entry_id,
                role,
                snippet,
                timestamp,
                score: rank,
            });
        }
    }
    
    // Sort by rank (FTS5 rank: lower is better)
    all_hits.sort_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));
    
    // Paginate
    let total_hits = all_hits.len();
    let start = page * page_size;
    let end = (start + page_size).min(total_hits);
    
    let hits = if start < total_hits {
        all_hits[start..end].to_vec()
    } else {
        vec![]
    };
    
    Ok(FullTextSearchResponse {
        hits,
        total_hits,
        has_more: end < total_hits,
    })
}
