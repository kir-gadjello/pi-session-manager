use serde_json::Value;
use std::fs;

pub async fn export_session(
    session_path: &str,
    format: &str,
    output_path: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(session_path)
        .map_err(|e| format!("Failed to read session file: {}", e))?;

    let export_content = match format {
        "json" => export_as_json(&content)?,
        "md" | "markdown" => export_as_markdown(&content)?,
        "html" => export_as_html(&content)?,
        _ => return Err(format!("Unsupported format: {}", format)),
    };

    fs::write(output_path, export_content)
        .map_err(|e| format!("Failed to write export file: {}", e))?;

    Ok(())
}

fn export_as_json(content: &str) -> Result<String, String> {
    // Parse and pretty print JSONL as JSON array
    let entries: Vec<Value> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))
}

fn export_as_markdown(content: &str) -> Result<String, String> {
    let mut md = String::new();
    let mut session_name = String::from("Session Export");
    let mut session_date = String::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            let entry_type = entry["type"].as_str().unwrap_or("unknown");

            if entry_type == "session" {
                if let Some(name) = entry["name"].as_str() {
                    session_name = name.to_string();
                }
                if let Some(ts) = entry["timestamp"].as_str() {
                    session_date = ts.to_string();
                }
                md.push_str(&format!("# {}\n\n", session_name));
                md.push_str(&format!("**Date:** {}\n\n", session_date));
                md.push_str("---\n\n");
            }

            if entry_type == "message" {
                if let Some(message) = entry.get("message") {
                    let role = message["role"].as_str().unwrap_or("unknown");
                    let timestamp = entry["timestamp"].as_str().unwrap_or("");

                    let role_label = match role {
                        "user" => "**User**",
                        "assistant" => "**Assistant**",
                        _ => &format!("**{}**", role),
                    };

                    md.push_str(&format!("{} *{}*\n\n", role_label, timestamp));

                    if let Some(content_arr) = message["content"].as_array() {
                        for item in content_arr {
                            if let Some(text) = item["text"].as_str() {
                                md.push_str(text);
                                md.push_str("\n\n");
                            }
                        }
                    }

                    md.push_str("---\n\n");
                }
            }
        }
    }

    Ok(md)
}

fn export_as_html(content: &str) -> Result<String, String> {
    let mut messages = Vec::new();
    let mut session_name = String::from("Session Export");

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            let entry_type = entry["type"].as_str().unwrap_or("unknown");

            if entry_type == "session" {
                if let Some(name) = entry["name"].as_str() {
                    session_name = name.to_string();
                }
            }

            if entry_type == "message" {
                if let Some(message) = entry.get("message") {
                    let role = message["role"].as_str().unwrap_or("unknown").to_string();
                    let timestamp = entry["timestamp"].as_str().unwrap_or("").to_string();

                    let mut text_parts = Vec::new();
                    if let Some(content_arr) = message["content"].as_array() {
                        for item in content_arr {
                            if let Some(text) = item["text"].as_str() {
                                text_parts.push(escape_html(text));
                            }
                        }
                    }

                    messages.push((role, timestamp, text_parts.join("")));
                }
            }
        }
    }

    let messages_html: String = messages
        .iter()
        .map(|(role, timestamp, text)| {
            let role_class = if role == "user" { "user-message" } else { "assistant-message" };
            format!(
                r#"<div class="message {}">
                    <div class="message-header">{} - {}</div>
                    <div class="message-content">{}</div>
                </div>"#,
                role_class, role, timestamp, text
            )
        })
        .collect();

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        :root {{
            --bg: #1e1e1e;
            --fg: #d4d4d4;
            --user-bg: #2d2d2d;
            --assistant-bg: #1e1e1e;
            --border: #3e3e3e;
            --accent: #569cd6;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: var(--bg);
            color: var(--fg);
            line-height: 1.6;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        h1 {{
            color: var(--accent);
            border-bottom: 1px solid var(--border);
            padding-bottom: 10px;
        }}
        .message {{
            margin: 20px 0;
            padding: 15px;
            border-radius: 8px;
            border: 1px solid var(--border);
        }}
        .user-message {{
            background: var(--user-bg);
        }}
        .assistant-message {{
            background: var(--assistant-bg);
        }}
        .message-header {{
            font-size: 12px;
            color: #888;
            margin-bottom: 10px;
            text-transform: uppercase;
        }}
        .message-content {{
            white-space: pre-wrap;
            word-wrap: break-word;
        }}
        pre {{
            background: #2d2d2d;
            padding: 10px;
            border-radius: 4px;
            overflow-x: auto;
        }}
        code {{
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
        session_name, session_name, messages_html
    ))
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\n', "<br>")
}