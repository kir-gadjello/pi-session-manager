---
id: "2026-01-30-Enhance search and add directory-based session grouping"
title: "Enhance search and add directory-based session grouping"
status: "done"
created: "2026-01-30"
updated: "2026-01-30"
category: "feature"
tags: ["search", "ui", "grouping"]
---

# Issue: Enhance search and add directory-based session grouping

## Goal

增强搜索功能，支持搜索会话名称和消息内容，并可切换搜索用户/Assistant消息；支持左侧边栏按目录分类显示会话并添加切换视图功能。

## 背景/问题

1. 搜索功能仅支持模糊搜索消息内容，无法搜索会话名称，也无法过滤消息角色
2. 左侧边栏按列表显示，无法按目录分类查看，不利于管理不同项目的会话

## 验收标准 (Acceptance Criteria)

- [x] 搜索面板支持 Name/Content 模式切换
- [x] Content 模式下支持 User/Assistant/All 角色过滤
- [x] Content 模式默认筛选用户消息（User）
- [x] 左侧边栏添加视图切换按钮（列表视图/目录视图）
- [x] 目录视图按 cwd 分组显示会话
- [x] 目录支持折叠/展开
- [x] 目录标题支持黏性滚动（sticky scroll）

## 实施阶段

### Phase 1: 规划和准备
- [x] 分析需求和依赖
- [x] 设计技术方案

### Phase 2: 执行
- [x] 修改 SearchPanel 组件，添加 Name/Content 切换和角色过滤
- [x] 修改后端 search.rs，支持 SearchMode 和 RoleFilter
- [x] 修改 commands.rs，接受新的搜索参数
- [x] 创建 SessionListByDirectory 组件
- [x] 修改 App.tsx，添加视图切换功能

### Phase 3: 验证
- [x] TypeScript 编译通过
- [x] Rust 编译通过
- [x] 应用正常启动

### Phase 4: 交付
- [x] 功能实现完成

## 关键决策

| 决策 | 理由 |
|------|------|
| 使用枚举类型 SearchMode 和 RoleFilter | 类型安全，便于扩展 |
| 默认角色过滤为 User | 用户通常更关心自己的消息 |
| 创建独立的 SessionListByDirectory 组件 | 保持原有组件简单，避免过度耦合 |

## 遇到的错误

| 日期 | 错误 | 解决方案 |
|------|------|---------|
| 2026-01-30 | tantivy_search.rs 调用 search_sessions 参数不匹配 | 添加 SearchMode 和 RoleFilter 参数 |

## 相关资源

- [ ] 相关文档: N/A
- [ ] 相关 Issue: N/A

## Notes

### 实施细节

1. **搜索功能增强**
   - `SearchPanel.tsx`: 添加 Name/Content 切换按钮，Content 模式下显示角色过滤按钮
   - `search.rs`: 新增 `SearchMode` 枚举和 `RoleFilter` 枚举
   - `search_sessions()`: 根据模式执行不同搜索逻辑

2. **目录分组显示**
   - `SessionListByDirectory.tsx`: 新组件，按 cwd 分组，支持折叠/展开
   - `App.tsx`: 添加 viewMode 状态和视图切换按钮

3. **用户体验优化**
   - 默认展开所有目录
   - 按修改时间排序会话
   - 目录名称智能提取（显示最后一级目录名）
   - 目录标题黏性滚动：滚动时目录标题固定在顶部

---

## 后续改进

- **黏性滚动**：目录标题使用 `sticky top-0` 实现黏性定位，方便查看当前目录

## Status 更新日志

- **[2026-01-30 13:05]**: 状态变更 → done，备注: 功能实现完成，编译通过
- **[2026-01-30 13:15]**: 添加目录标题黏性滚动功能