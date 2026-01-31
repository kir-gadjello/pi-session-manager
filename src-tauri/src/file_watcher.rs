use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::{Duration, SystemTime};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info, warn};

use crate::scanner;

/// 启动文件监听器
pub fn start_file_watcher(sessions_dir: PathBuf, app_handle: AppHandle) -> Result<(), String> {
    info!("Starting file watcher for: {:?}", sessions_dir);

    // 创建事件通道
    let (tx, rx) = channel();

    // 最后触发时间（用于额外的防抖）
    let last_trigger = Arc::new(Mutex::new(SystemTime::now()));

    // 创建防抖动的监听器（5秒防抖，避免频繁触发）
    let mut debouncer = new_debouncer(
        Duration::from_secs(5),
        None,
        move |result: DebounceEventResult| {
            if let Err(e) = tx.send(result) {
                error!("Failed to send file event: {:?}", e);
            }
        },
    )
    .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    // 监听 sessions 目录
    debouncer
        .watcher()
        .watch(&sessions_dir, RecursiveMode::Recursive)
        .map_err(|e| format!("Failed to watch directory: {}", e))?;

    info!("File watcher started successfully (5s debounce)");

    // 启动后台线程处理文件事件
    std::thread::spawn(move || {
        while let Ok(result) = rx.recv() {
            match result {
                Ok(events) => {
                    // 检查是否有 .jsonl 文件变化
                    let has_jsonl_changes = events.iter().any(|event| {
                        event.paths.iter().any(|path| {
                            // 只监听 .jsonl 文件，忽略 .db 等其他文件
                            path.extension()
                                .map(|ext| ext == "jsonl")
                                .unwrap_or(false)
                        })
                    });

                    if !has_jsonl_changes {
                        continue;
                    }

                    // 额外的防抖：确保距离上次触发至少 10 秒
                    let mut last = last_trigger.lock().unwrap();
                    let now = SystemTime::now();
                    if let Ok(elapsed) = now.duration_since(*last) {
                        if elapsed.as_secs() < 10 {
                            warn!("Skipping notification, too soon after last trigger ({:?})", elapsed);
                            continue;
                        }
                    }
                    *last = now;
                    drop(last);

                    info!("Detected .jsonl file changes, notifying frontend...");
                    
                    // 发送事件到前端
                    if let Err(e) = app_handle.emit("sessions-changed", ()) {
                        error!("Failed to emit event: {}", e);
                    }
                }
                Err(errors) => {
                    for error in errors {
                        error!("File watcher error: {:?}", error);
                    }
                }
            }
        }
    });

    // 将 debouncer 泄漏到静态生命周期，保持运行
    Box::leak(Box::new(debouncer));

    Ok(())
}
