# Imports

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Dependency Graph

```mermaid
graph TD
    root[root] --> settings[settings]
    root[root] --> contexts[contexts]
    root[root] --> hooks[hooks]
    root[root] --> styles[styles]
    root[root] --> utils[utils]
    root[root] --> utils[utils]
    root[root] --> utils[utils]
    root[root] --> utils[utils]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> components[components]
    root[root] --> components[components]
    root[root] --> components[components]
    root[root] --> components[components]
    root[root] --> components[components]
    root[root] --> components[components]
    root[root] --> components[components]
    root[root] --> components[components]
    root[root] --> settings[settings]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> dashboard[dashboard]
    root[root] --> hooks[hooks]
    root[root] --> hooks[hooks]
    root[root] --> hooks[hooks]
    root[root] --> hooks[hooks]
    root[root] --> hooks[hooks]
    root[root] --> hooks[hooks]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> _[.]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> sections[sections]
    root[root] --> _[.]
    root[root] --> diffs[diffs]
    root[root] --> _tanstack[@tanstack]
    root[root] --> api[api]
    root[root] --> api[api]
    root[root] --> _vitejs[@vitejs]
    root[root] --> chrono[chrono]
    root[root] --> commands[commands]
    root[root] --> date_fns[date-fns]
    root[root] --> highlight_js[highlight.js]
    root[root] --> lucide_react[lucide-react]
    root[root] --> notify[notify]
    root[root] --> notify_debouncer_full[notify_debouncer_full]
    root[root] --> pi_session_manager[pi_session_manager]
    root[root] --> react[react]
    root[root] --> react_dom[react-dom]
    root[root] --> react_dom[react-dom]
    root[root] --> react_i18next[react-i18next]
    root[root] --> rusqlite[rusqlite]
    root[root] --> serde[serde]
    root[root] --> serde_json[serde_json]
    root[root] --> std[std]
    root[root] --> super[super]
    root[root] --> tempfile[tempfile]
    root[root] --> tokio[tokio]
    root[root] --> tracing[tracing]
```

## Internal Dependencies

Dependencies within this module:

- `config`
- `export`
- `file_watcher`
- `models`
- `scanner`
- `scanner_scheduler`
- `search`
- `session_parser`
- `sqlite_cache`
- `stats`
- `tantivy_search`
- `tauri`
- `vite`

## External Dependencies

Dependencies from other modules:

- `../components/settings/types`
- `../contexts/SessionViewContext`
- `../hooks/useDemoMode`
- `../styles/session.css`
- `../utils/format`
- `../utils/markdown`
- `../utils/search`
- `../utils/session`
- `./App.tsx`
- `./AssistantMessage`
- `./BashExecution`
- `./BranchSummary`
- `./CodeBlock`
- `./Compaction`
- `./CustomMessage`
- `./EditExecution`
- `./ExpandableOutput`
- `./GenericToolCall`
- `./MarkdownContent`
- `./ModelChange`
- `./OpenInBrowserButton`
- `./OpenInTerminalButton`
- `./ReadExecution`
- `./SessionBadge`
- `./SessionHeader`
- `./SessionList`
- `./SessionTree`
- `./Skeleton`
- `./SystemPromptDialog`
- `./ThinkingBlock`
- `./ToolCallList`
- `./TreeNode`
- `./UserMessage`
- `./WriteExecution`
- `./components/Dashboard`
- `./components/ExportDialog`
- `./components/FavoritesPanel`
- `./components/ProjectList`
- `./components/RenameDialog`
- `./components/SessionList`
- `./components/SessionViewer`
- `./components/command`
- `./components/settings/SettingsPanel`
- `./dashboard/Achievements`
- `./dashboard/ActivityHeatmap`
- `./dashboard/ActivityTrend`
- `./dashboard/MessageDistribution`
- `./dashboard/ProductivityMetrics`
- `./dashboard/ProjectsChart`
- `./dashboard/RecentSessions`
- `./dashboard/SessionLengthChart`
- `./dashboard/StatCard`
- `./dashboard/TimeDistribution`
- `./dashboard/TokenStats`
- `./dashboard/TokenTrendChart`
- `./dashboard/TopModelsChart`
- `./dashboard/WeeklyComparison`
- `./hooks/useAppSettings`
- `./hooks/useFileWatcher`
- `./hooks/useKeyboardShortcuts`
- `./hooks/useSessionActions`
- `./hooks/useSessionBadges`
- `./hooks/useSessions`
- `./i18n`
- `./index.css`
- `./plugins`
- `./sections/AdvancedSettings`
- `./sections/AppearanceSettings`
- `./sections/ExportSettings`
- `./sections/LanguageSettings`
- `./sections/ModelSettings`
- `./sections/PiConfigSettings`
- `./sections/SearchSettings`
- `./sections/SessionSettings`
- `./sections/TerminalSettings`
- `./types`
- `@pierre/diffs/react`
- `@tanstack/react-virtual`
- `@tauri-apps/api/core`
- `@tauri-apps/api/window`
- `@vitejs/plugin-react`
- `chrono`
- `commands`
- `date-fns`
- `highlight.js`
- `lucide-react`
- `notify`
- `notify_debouncer_full`
- `pi_session_manager`
- `react`
- `react-dom`
- `react-dom/client`
- `react-i18next`
- `rusqlite`
- `serde`
- `serde_json`
- `std`
- `super`
- `tempfile`
- `tokio`
- `tracing`

