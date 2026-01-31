# Diff 渲染修复总结

## 问题描述
页面中的 diff 渲染出现错误，日志显示大量重复的 diff 内容，并且行号显示混乱。

## 根本原因
1. **错误的 CSS 导入**：`src/index.css` 中尝试导入 `@pierre/diffs/styles.css`，但该包不导出此文件
2. **错误的组件使用**：使用了 `FileDiff` 而不是正确的 `MultiFileDiff` 组件
3. **API 不匹配**：使用了错误的属性结构（`oldFile.path/content` vs `oldFile.name/contents`）

## 修复内容

### 1. 移除错误的 CSS 导入
**文件**: `src/index.css`

```diff
- @import '@pierre/diffs/styles.css';
-
  @tailwind base;
```

**原因**: `@pierre/diffs` 包将样式内联在 JS 中，不需要单独导入 CSS 文件。

### 2. 更新为正确的组件和 API
**文件**: `src/components/EditExecution.tsx`

#### 导入更新
```typescript
import { MultiFileDiff, type FileContents } from '@pierre/diffs/react'
```

#### 使用正确的 API
```typescript
const oldFile: FileContents = {
  name: fileName,
  contents: parsed.oldText,
}

const newFile: FileContents = {
  name: fileName,
  contents: parsed.newText,
}

<MultiFileDiff
  oldFile={oldFile}
  newFile={newFile}
  options={{
    theme: { dark: 'pierre-dark', light: 'pierre-light' },
    themeType: 'system', // 自动跟随系统主题
    diffStyle: 'split',
    overflow: 'wrap', // 启用换行
  }}
/>
```

### 3. 新增功能

#### 3.1 夜间主题支持
通过 `themeType: 'system'` 选项，diff 组件会自动跟随系统主题切换：
- 系统为深色模式时使用 `pierre-dark` 主题
- 系统为浅色模式时使用 `pierre-light` 主题

#### 3.2 换行支持
通过 `overflow: 'wrap'` 选项，长行会自动换行而不是横向滚动。

#### 3.3 复制功能
添加了复制按钮，可以将 diff 内容复制到剪贴板：

```typescript
const [copied, setCopied] = useState(false)

const copyDiffToClipboard = async () => {
  if (!diff) return
  
  try {
    await navigator.clipboard.writeText(diff)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  } catch (err) {
    console.error('Failed to copy diff to clipboard', err)
  }
}
```

UI 中添加了复制按钮，点击后会显示 ✓ 图标 2 秒钟作为反馈。

### 4. 改进的 diff 解析逻辑
优化了 Pi diff 格式的解析：
- 正确处理行号标记（`+`, `-`, 无标记）
- 保留空行
- 跳过省略标记（`...`）
- 移除了调试日志以提升性能

## 功能特性

### ✅ 已实现
1. **夜间主题**：自动跟随系统主题
2. **换行**：长行自动换行
3. **复制**：一键复制 diff 内容
4. **展开/折叠**：可以控制 diff 的显示/隐藏
5. **降级渲染**：如果解析失败，会显示彩色文本或原始内容

### 🎨 UI 改进
- 展开/折叠按钮和复制按钮并排显示
- 复制成功后显示视觉反馈
- 按钮样式与应用整体风格一致

## 测试建议
1. 打开包含 edit 工具调用的会话
2. 验证 diff 是否正确渲染（分屏显示，左侧旧内容，右侧新内容）
3. 测试复制功能是否正常工作
4. 切换系统主题，验证 diff 主题是否自动切换
5. 检查长行是否正确换行

## 相关文件
- `src/components/EditExecution.tsx` - 主要修改
- `src/index.css` - 移除错误的导入
- `src/components/ToolCallList.tsx` - 传递 expanded 属性

## 参考资料
- [@pierre/diffs 文档](https://diffs.com)
- [MultiFileDiff API](https://diffs.com/docs/components/multi-file-diff)
