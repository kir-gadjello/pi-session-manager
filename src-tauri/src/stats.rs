use crate::models::SessionInfo;
use crate::session_parser::parse_session_details;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{Datelike, Timelike, Weekday};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub total_messages: usize,
    pub user_messages: usize,
    pub assistant_messages: usize,
    pub total_tokens: usize,
    pub sessions_by_project: HashMap<String, usize>,
    pub sessions_by_model: HashMap<String, usize>,
    pub messages_by_date: HashMap<String, usize>,
    pub messages_by_hour: HashMap<String, usize>,
    pub messages_by_day_of_week: HashMap<String, usize>,
    pub average_messages_per_session: f32,
    pub heatmap_data: Vec<HeatmapPoint>,
    pub time_distribution: Vec<TimeDistributionPoint>,
    pub token_details: TokenDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDetails {
    pub total_input: usize,
    pub total_output: usize,
    pub total_cache_read: usize,
    pub total_cache_write: usize,
    pub total_cost: f64,
    pub tokens_by_model: HashMap<String, ModelTokenStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTokenStats {
    pub input: usize,
    pub output: usize,
    pub cache_read: usize,
    pub cache_write: usize,
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapPoint {
    pub date: String,
    pub level: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDistributionPoint {
    pub hour: usize,
    pub message_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyActivity {
    pub date: String,
    pub message_count: usize,
    pub session_count: usize,
}

pub fn calculate_stats(sessions: &[SessionInfo]) -> SessionStats {
    let total_sessions = sessions.len();

    println!("Calculating stats for {} sessions", total_sessions);

    let mut sessions_by_project: HashMap<String, usize> = HashMap::new();
    let mut sessions_by_model: HashMap<String, usize> = HashMap::new();
    let mut messages_by_date: HashMap<String, usize> = HashMap::new();
    let mut messages_by_hour: HashMap<String, usize> = HashMap::new();
    let mut messages_by_day_of_week: HashMap<String, usize> = HashMap::new();

    // Message counts
    let mut total_user_messages = 0usize;
    let mut total_assistant_messages = 0usize;
    let mut total_messages = 0usize;

    // Token statistics
    let mut total_input = 0usize;
    let mut total_output = 0usize;
    let mut total_cache_read = 0usize;
    let mut total_cache_write = 0usize;
    let mut total_cost = 0.0f64;
    let tokens_by_model: HashMap<String, ModelTokenStats> = HashMap::new();

    for session in sessions {
        // Extract project from cwd
        let project = extract_project_name(&session.cwd);
        *sessions_by_project.entry(project).or_insert(0) += 1;

        // Parse session file for detailed stats
        if let Ok(content) = std::fs::read_to_string(&session.path) {
            let session_stats = parse_session_details(&content);
            // Update model usage
            for model in &session_stats.models {
                *sessions_by_model.entry(model.clone()).or_insert(0) += 1;
            }

            // Update message counts
            total_user_messages += session_stats.user_messages;
            total_assistant_messages += session_stats.assistant_messages;
            total_messages += session_stats.user_messages + session_stats.assistant_messages;

            // Update token counts
            total_input += session_stats.input_tokens as usize;
            total_output += session_stats.output_tokens as usize;
            total_cache_read += session_stats.cache_read_tokens as usize;
            total_cache_write += session_stats.cache_write_tokens as usize;
            total_cost += session_stats.input_cost + session_stats.output_cost + session_stats.cache_read_cost + session_stats.cache_write_cost;

            // Messages by date (use session modified time as fallback)
            let date = session.modified.format("%Y-%m-%d").to_string();
            *messages_by_date.entry(date.clone()).or_insert(0) += session_stats.user_messages + session_stats.assistant_messages;

            // Messages by hour
            let hour = session.modified.hour();
            *messages_by_hour.entry(hour.to_string()).or_insert(0) += session_stats.user_messages + session_stats.assistant_messages;

            // Messages by day of week
            let weekday = session.modified.weekday();
            let day_name = match weekday {
                Weekday::Mon => "Monday",
                Weekday::Tue => "Tuesday",
                Weekday::Wed => "Wednesday",
                Weekday::Thu => "Thursday",
                Weekday::Fri => "Friday",
                Weekday::Sat => "Saturday",
                Weekday::Sun => "Sunday",
            };
            *messages_by_day_of_week.entry(day_name.to_string()).or_insert(0) += session_stats.user_messages + session_stats.assistant_messages;
        } else {
            // Fallback if parsing fails
            *sessions_by_model.entry("unknown".to_string()).or_insert(0) += 1;
            total_messages += session.message_count;

            // Messages by date (use session modified time)
            let date = session.modified.format("%Y-%m-%d").to_string();
            *messages_by_date.entry(date.clone()).or_insert(0) += session.message_count;

            // Messages by hour
            let hour = session.modified.hour();
            *messages_by_hour.entry(hour.to_string()).or_insert(0) += session.message_count;

            // Messages by day of week
            let weekday = session.modified.weekday();
            let day_name = match weekday {
                Weekday::Mon => "Monday",
                Weekday::Tue => "Tuesday",
                Weekday::Wed => "Wednesday",
                Weekday::Thu => "Thursday",
                Weekday::Fri => "Friday",
                Weekday::Sat => "Saturday",
                Weekday::Sun => "Sunday",
            };
            *messages_by_day_of_week.entry(day_name.to_string()).or_insert(0) += session.message_count;
        }
    }

    let average_messages_per_session = if total_sessions > 0 {
        total_messages as f32 / total_sessions as f32
    } else {
        0.0
    };

    // Generate heatmap data (last 365 days)
    let heatmap_data = generate_heatmap_data(&messages_by_date);

    // Generate time distribution
    let time_distribution = generate_time_distribution(&messages_by_hour);

    println!("Stats: {} user messages, {} assistant messages, {} total tokens", total_user_messages, total_assistant_messages, total_input + total_output);

    SessionStats {
        total_sessions,
        total_messages,
        user_messages: total_user_messages,
        assistant_messages: total_assistant_messages,
        total_tokens: total_input + total_output,
        sessions_by_project,
        sessions_by_model,
        messages_by_date,
        messages_by_hour,
        messages_by_day_of_week,
        average_messages_per_session,
        heatmap_data,
        time_distribution,
        token_details: TokenDetails {
            total_input,
            total_output,
            total_cache_read,
            total_cache_write,
            total_cost,
            tokens_by_model,
        },
    }
}

fn extract_project_name(cwd: &str) -> String {
    cwd.split('/')
        .last()
        .unwrap_or("unknown")
        .to_string()
}

fn generate_heatmap_data(messages_by_date: &HashMap<String, usize>) -> Vec<HeatmapPoint> {
    let mut data = Vec::new();
    let now = chrono::Utc::now();
    let days_ago = 365;

    // Find max messages for normalization
    let max_messages = messages_by_date.values().copied().max().unwrap_or(1);

    for i in 0..=days_ago {
        let date = now - chrono::Duration::days(i);
        let date_str = date.format("%Y-%m-%d").to_string();
        let message_count = messages_by_date.get(&date_str).copied().unwrap_or(0);

        // Calculate activity level (0-5)
        let level = if message_count == 0 {
            0
        } else {
            ((message_count as f32 / max_messages as f32) * 5.0).round() as usize
        };

        data.push(HeatmapPoint {
            date: date_str,
            level,
        });
    }

    data.reverse();
    data
}

fn generate_time_distribution(messages_by_hour: &HashMap<String, usize>) -> Vec<TimeDistributionPoint> {
    let mut distribution = Vec::new();

    for hour in 0..24 {
        let message_count = messages_by_hour.get(&hour.to_string()).copied().unwrap_or(0);
        distribution.push(TimeDistributionPoint {
            hour,
            message_count,
        });
    }

    distribution
}

pub fn get_activity_timeline(sessions: &[SessionInfo]) -> Vec<DailyActivity> {
    let mut activity: HashMap<String, (usize, usize)> = HashMap::new();

    for session in sessions {
        let date = session.modified.format("%Y-%m-%d").to_string();
        let entry = activity.entry(date).or_insert((0, 0));
        entry.0 += session.message_count;
        entry.1 += 1;
    }

    let mut timeline: Vec<DailyActivity> = activity
        .into_iter()
        .map(|(date, (messages, sessions))| DailyActivity {
            date,
            message_count: messages,
            session_count: sessions,
        })
        .collect();

    timeline.sort_by(|a, b| a.date.cmp(&b.date));
    timeline
}