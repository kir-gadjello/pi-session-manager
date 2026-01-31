use crate::models::{Match, SearchResult, SessionInfo};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchMode {
    Name,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoleFilter {
    All,
    User,
    Assistant,
}

pub fn search_sessions(
    sessions: &[SessionInfo],
    query: &str,
    search_mode: SearchMode,
    role_filter: RoleFilter,
    include_tools: bool,
) -> Vec<SearchResult> {
    if query.trim().is_empty() {
        return vec![];
    }

    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut results = Vec::new();

    for session in sessions {
        if search_mode == SearchMode::Name {
            // Search in session name and first message
            if matches_session_name(session, &query_words) {
                results.push(SearchResult {
                    session_id: session.id.clone(),
                    session_path: session.path.clone(),
                    session_name: session.name.clone(),
                    first_message: session.first_message.clone(),
                    matches: vec![],
                    score: 1.0,
                });
            }
        } else {
            // Search in message content
            let matches = find_matches(session, &query_words, role_filter, include_tools);
            if !matches.is_empty() {
                let score = calculate_score(&matches, &query_words);
                results.push(SearchResult {
                    session_id: session.id.clone(),
                    session_path: session.path.clone(),
                    session_name: session.name.clone(),
                    first_message: session.first_message.clone(),
                    matches,
                    score,
                });
            }
        }
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    results
}

fn matches_session_name(session: &SessionInfo, query_words: &[&str]) -> bool {
    if query_words.is_empty() {
        return false;
    }

    let name_lower = session.name.as_deref().unwrap_or("").to_lowercase();
    let first_msg_lower = session.first_message.to_lowercase();
    let combined = format!("{} {}", name_lower, first_msg_lower);

    // All words must match (AND logic) for more precise name search
    query_words.iter().all(|word| combined.contains(word))
}

fn find_matches(
    session: &SessionInfo,
    query_words: &[&str],
    role_filter: RoleFilter,
    include_tools: bool,
) -> Vec<Match> {
    if query_words.is_empty() {
        return vec![];
    }

    // Fast filter using all_messages_text (already in memory)
    let all_text_lower = session.all_messages_text.to_lowercase();
    let has_any_match = query_words.iter().any(|word| all_text_lower.contains(word));
    if !has_any_match {
        return vec![];
    }

    let mut matches = Vec::new();

    // Parse session entries to get detailed match information with roles
    let entries = parse_session_entries(session.path.as_str(), role_filter, include_tools);

    for entry in &entries {
        let content_lower = entry.content.to_lowercase();

        // Check if ANY query word matches in this entry (OR logic)
        let any_word_match = query_words.iter().any(|word| content_lower.contains(word));

        if any_word_match {
            // Find the position of the first matching word for snippet
            for word in query_words {
                if let Some(word_pos) = content_lower.find(word) {
                    let snippet_start = word_pos.saturating_sub(30);
                    let snippet_end = (word_pos + word.len() + 100).min(entry.content.len());
                    let snippet = entry.content[snippet_start..snippet_end].to_string();

                    matches.push(Match {
                        entry_id: entry.id.clone(),
                        role: entry.role.clone(),
                        snippet,
                        timestamp: entry.timestamp,
                    });
                    break; // Only add one snippet per entry
                }
            }
        }
    }

    matches.dedup_by(|a, b| a.entry_id == b.entry_id);
    matches.truncate(5);
    matches
}

struct MessageEntry {
    id: String,
    role: String,
    content: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

fn parse_session_entries(
    path: &str,
    role_filter: RoleFilter,
    include_tools: bool,
) -> Vec<MessageEntry> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read session file: {}", e))
        .unwrap_or_default();

    let mut entries = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            // Only process message entries
            if entry["type"] != "message" {
                continue;
            }

            if let Some(message) = entry.get("message") {
                let role = message["role"].as_str().unwrap_or("unknown");
                let timestamp_str = entry["timestamp"].as_str().unwrap_or("");
                let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                // Filter by role
                let include = match role_filter {
                    RoleFilter::All => true,
                    RoleFilter::User => role == "user",
                    RoleFilter::Assistant => role == "assistant",
                };

                if include {
                    let empty_array = vec![];
                    let content = message["content"].as_array().unwrap_or(&empty_array);

                    let mut text = String::new();
                    for item in content {
                        if let Some(text_content) = item.get("text") {
                            if let Some(s) = text_content.as_str() {
                                text.push_str(s);
                            }
                        }

                        // Include thinking content if include_tools is true
                        if include_tools {
                            if let Some(thinking) = item.get("thinking") {
                                if let Some(s) = thinking.as_str() {
                                    text.push_str(s);
                                }
                            }
                        }
                    }

                    if !text.is_empty() {
                        entries.push(MessageEntry {
                            id: entry["id"].as_str().unwrap_or("").to_string(),
                            role: role.to_string(),
                            content: text,
                            timestamp,
                        });
                    }
                }
            }
        }
    }

    entries
}

fn get_filtered_session_content(path: &str, role_filter: RoleFilter) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read session file: {}", e))?;

    let mut full_text = String::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            if entry["type"] == "message" {
                if let Some(message) = entry.get("message") {
                    let role = message["role"].as_str().unwrap_or("");

                    // Filter by role
                    let include = match role_filter {
                        RoleFilter::All => true,
                        RoleFilter::User => role == "user",
                        RoleFilter::Assistant => role == "assistant",
                    };

                    if include {
                        let empty_array = vec![];
                        let content = message["content"].as_array().unwrap_or(&empty_array);

                        for item in content {
                            if let Some(text) = item.get("text") {
                                if let Some(s) = text.as_str() {
                                    full_text.push_str(s);
                                    full_text.push_str("\n");
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(full_text)
}

fn role_to_string(role_filter: RoleFilter) -> String {
    match role_filter {
        RoleFilter::All => "all".to_string(),
        RoleFilter::User => "user".to_string(),
        RoleFilter::Assistant => "assistant".to_string(),
    }
}

fn get_full_session_content(path: &str) -> Result<String, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read session file: {}", e))?;

    let mut full_text = String::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            if entry["type"] == "message" {
                if let Some(message) = entry.get("message") {
                    let empty_array = vec![];
                    let content = message["content"].as_array().unwrap_or(&empty_array);

                    for item in content {
                        if let Some(text) = item.get("text") {
                            if let Some(s) = text.as_str() {
                                full_text.push_str(s);
                                full_text.push_str("\n");
                            }
                        }

                        // Include thinking content
                        if let Some(thinking) = item.get("thinking") {
                            if let Some(s) = thinking.as_str() {
                                full_text.push_str(s);
                                full_text.push_str("\n");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(full_text)
}

fn calculate_score(matches: &[Match], query_words: &[&str]) -> f32 {
    if matches.is_empty() {
        return 0.0;
    }

    // Base score: number of matches
    let mut score = matches.len() as f32;

    // Bonus for exact matches (word boundaries)
    for m in matches {
        let snippet_lower = m.snippet.to_lowercase();
        for word in query_words {
            // Check for word boundary match
            let word_with_boundary = format!(" {} ", word);
            let start_with_boundary = format!("{} ", word);
            let end_with_boundary = format!(" {}", word);

            if snippet_lower.contains(&word_with_boundary)
                || snippet_lower.starts_with(&start_with_boundary)
                || snippet_lower.ends_with(&end_with_boundary)
            {
                score += 0.5;
            }
        }
    }

    score
}