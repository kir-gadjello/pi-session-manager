use crate::models::{SessionEntry, SessionInfo};
use crate::scanner;
use crate::search;
use crate::export;
use crate::stats;
use crate::config;
use crate::sqlite_cache;
use serde_json::Value;
use std::fs;

#[tauri::command]
pub async fn scan_sessions() -> Result<Vec<SessionInfo>, String> {
    scanner::scan_sessions().await
}

#[tauri::command]
pub async fn read_session_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read session file: {}", e))
}

#[tauri::command]
pub async fn get_session_entries(path: String) -> Result<Vec<SessionEntry>, String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read session file: {}", e))?;

    let mut entries = Vec::new();

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(value) = serde_json::from_str::<Value>(line) {
            let entry_type = value["type"].as_str().unwrap_or("unknown").to_string();
            let id = value["id"].as_str().unwrap_or("").to_string();
            let parent_id = value["parentId"].as_str().map(|s| s.to_string());
            let timestamp_str = value["timestamp"].as_str().unwrap_or("");

            let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            let message = value.get("message").and_then(|m| serde_json::from_value(m.clone()).ok());

            entries.push(SessionEntry {
                entry_type,
                id,
                parent_id,
                timestamp,
                message,
            });
        }
    }

    Ok(entries)
}

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

    Ok(search::search_sessions(&sessions, &query, mode, role, include_tools))
}

#[tauri::command]
pub async fn delete_session(path: String) -> Result<(), String> {
    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete session: {}", e))
}

#[tauri::command]
pub async fn export_session(
    path: String,
    format: String,
    output_path: String,
) -> Result<(), String> {
    export::export_session(&path, &format, &output_path).await
}

#[tauri::command]
pub async fn rename_session(path: String, new_name: String) -> Result<(), String> {
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read session file: {}", e))?;

    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut name_updated = false;

    for line in &mut lines {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(mut value) = serde_json::from_str::<Value>(line) {
            if value["type"] == "session_info" || value["type"] == "session" {
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("name".to_string(), serde_json::Value::String(new_name.clone()));
                    *line = serde_json::to_string(&value)
                        .map_err(|e| format!("Failed to serialize: {}", e))?;
                    name_updated = true;
                    break;
                }
            }
        }
    }

    // If no session_info entry found, we need to add one
    if !name_updated {
        let session_info = serde_json::json!({
            "type": "session_info",
            "name": new_name,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        lines.push(serde_json::to_string(&session_info)
            .map_err(|e| format!("Failed to serialize: {}", e))?);
    }

    fs::write(&path, lines.join("\n"))
        .map_err(|e| format!("Failed to write session file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_session_stats(sessions: Vec<SessionInfo>) -> Result<stats::SessionStats, String> {
    Ok(stats::calculate_stats(&sessions))
}

/// 在终端中打开会话
/// 默认使用 iTerm2，后续支持配置其他终端
#[tauri::command]
pub async fn open_session_in_terminal(
    path: String,
    terminal: Option<String>,
    pi_path: Option<String>,
) -> Result<(), String> {
    use std::process::Command;

    let terminal = terminal.unwrap_or_else(|| "iterm2".to_string());
    let pi_cmd = pi_path.unwrap_or_else(|| "pi".to_string());

    let result = match terminal.as_str() {
        "iterm2" => {
            // iTerm2 AppleScript 命令
            let script = format!(
                r#"tell application "iTerm"
    activate
    set newWindow to (create window with default profile)
    tell current session of newWindow
        write text "{} --session {}"
    end tell
end tell"#,
                pi_cmd,
                path.replace("\"", "\\\"")
            );
            Command::new("osascript").arg("-e").arg(script).spawn()
        }
        "terminal" => {
            // Terminal.app AppleScript 命令
            let script = format!(
                r#"tell application "Terminal"
    activate
    do script "{} --session {}"
end tell"#,
                pi_cmd,
                path.replace("\"", "\\\"")
            );
            Command::new("osascript").arg("-e").arg(script).spawn()
        }
        "vscode" => {
            // VS Code 终端
            Command::new("code")
                .args(["--new-window", "--exec", &format!("{} --session {}", pi_cmd, path)])
                .spawn()
        }
        _ => {
            return Err(format!("Unsupported terminal: {}", terminal));
        }
    };

    result.map_err(|e| format!("Failed to open terminal: {}", e))?;

    Ok(())
}

/// 在浏览器中打开会话
/// 流程：导出为 HTML 到临时目录 -> 使用系统命令打开
#[tauri::command]
pub async fn open_session_in_browser(path: String) -> Result<(), String> {
    use std::process::Command;

    // 生成临时文件路径
    let temp_dir = std::env::temp_dir();
    let session_id = std::path::Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("session");
    let temp_html_path = temp_dir.join(format!("pi_session_{}.html", session_id));
    let temp_html_path_str = temp_html_path.to_string_lossy().to_string();

    // 1. 导出会话为 HTML
    export::export_session(&path, "html", &temp_html_path_str)
        .await
        .map_err(|e| format!("Failed to export session: {}", e))?;

    // 2. 使用系统命令打开 HTML 文件
    let result = if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(&temp_html_path_str)
            .spawn()
    } else if cfg!(target_os = "linux") {
        Command::new("xdg-open")
            .arg(&temp_html_path_str)
            .spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", &temp_html_path_str])
            .spawn()
    } else {
        return Err("Unsupported operating system".to_string());
    };

    result.map_err(|e| format!("Failed to open browser: {}", e))?;

    Ok(())
}

/// 扫描可用的 skills
#[tauri::command]
pub async fn scan_skills() -> Result<Vec<SkillInfo>, String> {
    use std::fs;

    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let skills_dir = home_dir.join(".pi/agent/skills");

    let mut skills = Vec::new();

    if let Ok(entries) = fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // 检查是否有 SKILL.md 文件
                let skill_md = path.join("SKILL.md");
                let has_skill_md = skill_md.exists();

                // 尝试读取 SKILL.md 的第一行作为描述
                let description = if has_skill_md {
                    fs::read_to_string(&skill_md)
                        .ok()
                        .and_then(|content| content.lines().next().map(|s| s.to_string()))
                        .unwrap_or_default()
                } else {
                    String::new()
                };

                skills.push(SkillInfo {
                    name: name.clone(),
                    path: format!("skills/{}/SKILL.md", name),
                    description,
                    enabled: true, // 默认启用，后续从 settings.json 读取
                });
            }
        }
    }

    // 按名称排序
    skills.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(skills)
}

/// 扫描可用的 prompts
#[tauri::command]
pub async fn scan_prompts() -> Result<Vec<PromptInfo>, String> {
    use std::fs;

    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let prompts_dir = home_dir.join(".pi/agent/prompts");

    let mut prompts = Vec::new();

    if let Ok(entries) = fs::read_dir(&prompts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                let name = path.file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // 读取第一行作为描述
                let description = fs::read_to_string(&path)
                    .ok()
                    .and_then(|content| content.lines().next().map(|s| s.trim_start_matches("# ").to_string()))
                    .unwrap_or_default();

                prompts.push(PromptInfo {
                    name: name.clone(),
                    path: format!("prompts/{}.md", name),
                    description,
                    enabled: true,
                });
            }
        }
    }

    // 按名称排序
    prompts.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(prompts)
}

/// 读取 Pi 的 settings.json
#[tauri::command]
pub async fn load_pi_settings() -> Result<PiSettings, String> {
    use std::fs;

    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let settings_path = home_dir.join(".pi/agent/settings.json");

    if !settings_path.exists() {
        // 返回默认设置
        return Ok(PiSettings {
            skills: Vec::new(),
            prompts: Vec::new(),
            extensions: Vec::new(),
        });
    }

    let content = fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings: {}", e))?;

    let skills = json.get("skills")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let prompts = json.get("prompts")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let extensions = json.get("extensions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    Ok(PiSettings {
        skills,
        prompts,
        extensions,
    })
}

/// 保存 Pi 的 settings.json
#[tauri::command]
pub async fn save_pi_settings(settings: PiSettings) -> Result<(), String> {
    use std::fs;

    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let settings_path = home_dir.join(".pi/agent/settings.json");

    // 读取现有设置
    let mut json: serde_json::Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings: {}", e))?
    } else {
        serde_json::json!({})
    };

    // 更新设置
    json["skills"] = serde_json::json!(settings.skills);
    json["prompts"] = serde_json::json!(settings.prompts);
    json["extensions"] = serde_json::json!(settings.extensions);

    // 写回文件
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SkillInfo {
    pub name: String,
    pub path: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PromptInfo {
    pub name: String,
    pub path: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct PiSettings {
    pub skills: Vec<String>,
    pub prompts: Vec<String>,
    pub extensions: Vec<String>,
}

/// 模型信息
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ModelInfo {
    pub provider: String,
    pub model: String,
    pub available: bool,
    pub tested: bool,
    pub last_test_time: Option<String>,
    pub response_time: Option<f64>,
    pub status: String,
}

/// 模型测试结果
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ModelTestResult {
    pub provider: String,
    pub model: String,
    pub time: f64,
    pub output: String,
    pub status: String,
    pub error_msg: Option<String>,
}

/// 列出可用模型
#[tauri::command]
pub async fn list_models(search: Option<String>) -> Result<Vec<ModelInfo>, String> {
    use std::process::Command;

    let mut args = vec!["--list-models".to_string()];
    if let Some(query) = search {
        args.push(query);
    }

    let output = Command::new("pi")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to execute pi --list-models: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("pi --list-models failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut models = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // 跳过表头
        if line.contains("provider") && line.contains("model") {
            continue;
        }

        // 解析表格行，格式: provider   model   context  max-out  thinking  images
        // 我们只需要前两列
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let provider = parts[0].to_string();
            let model = parts[1].to_string();

            models.push(ModelInfo {
                provider,
                model,
                available: true,
                tested: false,
                last_test_time: None,
                response_time: None,
                status: "ready".to_string(),
            });
        }
    }

    Ok(models)
}

/// 测试单个模型 - 简化版，只测成功/失败和速度
#[tauri::command]
pub async fn test_model(provider: String, model: String, prompt: Option<String>) -> Result<ModelTestResult, String> {
    use std::process::{Command, Stdio};
    use std::time::Instant;

    let args = vec![
        "--provider", &provider,
        "--model", &model,
        "--no-tools",
        "--no-skills",
        "--no-extensions",
        "--no-session",
        "--print",
    ];

    let start_time = Instant::now();

    // 简单测试：使用 stdin 输入
    let output = Command::new("pi")
        .args(&args)
        .stdin(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute pi: {}", e))?;

    let duration = start_time.elapsed().as_secs_f64();

    // 简单判断：成功就返回 OK
    if output.status.success() {
        Ok(ModelTestResult {
            provider,
            model,
            time: duration,
            output: "OK".to_string(),
            status: "success".to_string(),
            error_msg: None,
        })
    } else {
        Ok(ModelTestResult {
            provider,
            model,
            time: duration,
            output: "Failed".to_string(),
            status: "error".to_string(),
            error_msg: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

/// 清理 ANSI 控制码和 TUI 输出
fn clean_ansi_output(text: &str) -> String {
    let mut result = String::new();
    let mut in_ansi = false;
    let mut skip_until_newline = false;

    for c in text.chars() {
        if c == '\x1b' {
            in_ansi = true;
            continue;
        }
        
        if in_ansi {
            if c.is_ascii_alphabetic() || c == '@' {
                in_ansi = false;
            }
            continue;
        }

        // 跳过 TUI 控制序列
        if c == '\x07' || c == '\r' {
            continue;
        }

        // 如果遇到某些 TUI 标记，跳过直到换行
        if skip_until_newline {
            if c == '\n' {
                skip_until_newline = false;
            }
            continue;
        }

        result.push(c);
    }

    // 移除多余的空行
    result.lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// 批量测试模型
#[tauri::command]
pub async fn test_models_batch(models: Vec<(String, String)>, prompt: Option<String>) -> Result<Vec<ModelTestResult>, String> {
    let mut results = Vec::new();

    for (provider, model) in models {
        match test_model(provider.clone(), model.clone(), prompt.clone()).await {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(ModelTestResult {
                    provider,
                    model,
                    time: 0.0,
                    output: String::new(),
                    status: "error".to_string(),
                    error_msg: Some(e),
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn search_sessions_fts(
    query: String,
    limit: usize,
) -> Result<Vec<SessionInfo>, String> {
    let config = config::load_config()?;
    let conn = sqlite_cache::init_db_with_config(&config)?;
    
    // 使用 FTS5 搜索获取路径
    let paths = sqlite_cache::search_fts5(&conn, &query, limit)?;
    
    // 从数据库获取完整的 SessionInfo
    let mut sessions = Vec::new();
    for path in paths {
        if let Some(session) = sqlite_cache::get_session(&conn, &path)? {
            sessions.push(session);
        }
    }
    
    Ok(sessions)
}
