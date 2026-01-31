# 导出功能说明

## 功能概述

项目支持三种导出格式：
- **HTML** - 使用 PI 的 `--export` 命令生成完整的 HTML 页面
- **Markdown** - 生成带格式的 Markdown 文档
- **JSON** - 导出为 JSON 数组格式

## 使用方式

### 1. 在应用中导出

1. 选择一个会话
2. 点击右上角的"导出"按钮
3. 在弹出的对话框中选择导出格式
4. 选择保存位置
5. 等待导出完成

### 2. 使用命令行测试

```bash
# 测试 HTML 导出
pi --export ~/.pi/agent/sessions/--path--/session.jsonl /tmp/output.html

# 测试 Markdown 导出
cargo test --test export_test

# 测试 JSON 导出
cargo test --test export_test
```

## 技术实现

### 后端 (Rust)

**文件位置**: `src-tauri/src/export.rs`

#### HTML 导出

使用 PI 的 `--export` 命令生成 HTML：

```rust
fn export_using_pi_command(session_path: &str, output_path: &str) -> Result<(), String> {
    let output = Command::new("pi")
        .arg("--export")
        .arg(session_path)
        .arg(output_path)
        .output()
        .map_err(|e| format!("Failed to execute pi command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Pi export command failed: {}", stderr));
    }

    Ok(())
}
```

#### Markdown 导出

解析 JSONL 文件并生成 Markdown 文档：

```rust
fn export_as_markdown(session_path: &str, output_path: &str) -> Result<(), String> {
    // 读取会话文件
    let content = fs::read_to_string(session_path)?;

    // 解析并生成 Markdown
    let mut md = String::new();
    // ... 处理会话数据

    // 写入输出文件
    fs::write(output_path, md)?;
    Ok(())
}
```

#### JSON 导出

将 JSONL 格式转换为 JSON 数组：

```rust
fn export_as_json(session_path: &str, output_path: &str) -> Result<(), String> {
    let content = fs::read_to_string(session_path)?;

    let entries: Vec<Value> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let json_content = serde_json::to_string_pretty(&entries)?;
    fs::write(output_path, json_content)?;
    Ok(())
}
```

### 前端 (React)

**文件位置**: `src/App.tsx`

```typescript
const handleExportSession = async (format: 'html' | 'md' | 'json') => {
  if (!selectedSession) return

  // 打开保存对话框
  const filePath = await save({
    filters: [{ name: format.toUpperCase(), extensions: [extension] }],
    defaultPath: `${selectedSession.name || 'session'}.${extension}`
  })

  if (!filePath) return

  // 调用 Tauri 命令
  await invoke('export_session', {
    path: selectedSession.path,
    format,
    outputPath: filePath
  })
}
```

## 测试

运行导出功能测试：

```bash
cargo test --test export_test
```

测试包括：
- ✅ HTML 导出测试
- ✅ Markdown 导出测试
- ✅ JSON 导出测试

## 故障排查

### 导出失败

1. 检查 PI 命令是否可用：
   ```bash
   pi --export --help
   ```

2. 检查会话文件路径是否正确

3. 查看控制台日志获取详细错误信息

### HTML 导出样式问题

HTML 导出使用 PI 内置的模板，样式文件位于：
```
~/.local/share/nvm/v23.11.1/lib/node_modules/@mariozechner/pi-coding-agent/dist/core/export-html/
```

包含：
- `template.html` - HTML 模板
- `template.css` - 样式文件
- `template.js` - JavaScript 逻辑
- `vendor/` - 第三方库（marked.js, highlight.js）

## 配置

导出设置可在应用的设置面板中配置：

- 默认导出格式
- 包含元数据
- 包含时间戳

**文件位置**: `src/components/settings/sections/ExportSettings.tsx`

## 国际化

支持中英文界面：

**文件位置**:
- `src/i18n/locales/en-US/export.ts`
- `src/i18n/locales/zh-CN/export.ts`