# Pi Session Manager

A Tauri2 desktop application for managing and searching Pi Agent sessions. Built with Rust backend and React frontend.

## Features

### Core Features
- **Session List**: Browse all Pi sessions from `~/.pi/agent/sessions/`
- **Full-text Search**: Search across user messages and AI responses
- **Session Viewer**: View session details with embedded Pi HTML template
- **Dark Mode**: Beautiful dark mode UI with Tailwind-like styling

### Session Management
- **Delete Sessions**: Remove unwanted sessions with confirmation
- **Export Sessions**: Export to HTML, Markdown, or JSON formats
- **Rename Sessions**: Custom session names for easy identification

### Analytics
- **Statistics Dashboard**: View session statistics and metrics
- **Message Breakdown**: See user vs assistant message distribution
- **Top Projects**: Most frequently worked on projects

### UX Improvements
- **Keyboard Shortcuts**: Efficient navigation with hotkeys
- **Virtual Scrolling**: Smooth performance with large session lists
- **Search Results**: Match count badges for better search feedback
- **Internationalization (i18n)**: Support for multiple languages (English, 简体中文)

## Keyboard Shortcuts

- `Cmd/Ctrl + R` - Refresh session list
- `Cmd/Ctrl + F` - Focus search input
- `Cmd/Ctrl + Shift + S` - Open statistics panel
- `Esc` - Clear selection / Close dialogs

## Internationalization (i18n)

The application supports multiple languages:

- **English** (en-US) - Default language
- **简体中文** (zh-CN)

### Adding a New Language

1. Create a new translation file in `src/i18n/locales/`:
   ```typescript
   // src/i18n/locales/your-lang.ts
   export const yourLang = {
     common: {
       // translations
     },
     // other namespaces
   } as const
   ```

2. Add the language to `src/i18n/config.ts`:
   ```typescript
   resources: {
     en: { translation: enUS },
     zh: { translation: zhCN },
     yourLangCode: { translation: yourLang },
   }
   ```

3. Update `LanguageSwitcher.tsx` to include the new language option

### Translation Namespaces

- `common` - Common UI elements (buttons, labels)
- `app` - Application-level text
- `session` - Session-related text
- `search` - Search interface
- `export` - Export dialog
- `rename` - Rename dialog
- `project` - Project list
- `stats` - Statistics dashboard
- `dashboard` - Dashboard components

## Development

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### Installation

```bash
cd ~/Dev/AI/pi-session-manager
npm install
```

### Running

```bash
npm run tauri:dev
```

### Building

```bash
npm run tauri:build
```

## Project Structure

```
pi-session-manager/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # React components
│   │   ├── SessionList.tsx
│   │   ├── SessionViewer.tsx
│   │   ├── SearchPanel.tsx
│   │   ├── ExportDialog.tsx
│   │   ├── RenameDialog.tsx
│   │   └── StatsPanel.tsx
│   ├── hooks/
│   │   └── useKeyboardShortcuts.ts
│   ├── App.tsx            # Main app
│   └── types.ts           # TypeScript types
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── commands.rs    # Tauri IPC commands
│   │   ├── scanner.rs     # Session scanner
│   │   ├── search.rs      # Search engine
│   │   ├── export.rs      # Export functionality
│   │   ├── stats.rs       # Statistics calculation
│   │   ├── models.rs      # Data models
│   │   └── lib.rs         # Library entry point
│   └── Cargo.toml
└── tauri.conf.json        # Tauri configuration
```

## Architecture

### Backend (Rust)

- **Scanner**: Scans `~/.pi/agent/sessions/` for JSONL session files
- **Parser**: Parses JSONL format and extracts session metadata
- **Search**: Full-text search across session content
- **Export**: Export sessions to various formats
- **Stats**: Calculate statistics and analytics

### Frontend (React)

- **SessionList**: Virtual scrolling session list with delete support
- **SessionViewer**: Server-rendered HTML with export/rename buttons
- **SearchPanel**: Search input with tool call filtering
- **StatsPanel**: Statistics dashboard with visual metrics

## Data Source

Pi sessions are stored as JSONL files in `~/.pi/agent/sessions/`:

```
~/.pi/agent/sessions/
├── --Users-dengwenyu-project-a--/
│   └── 2026-01-30T12-00-00-000Z_uuid.jsonl
└── --Users-dengwenyu-project-b--/
    └── 2026-01-30T13-00-00-000Z_uuid.jsonl
```

Each `.jsonl` file contains:
- Session header (type: "session")
- Message entries (type: "message")
- Metadata entries (type: "session_info", "compaction", etc.)

## Dependencies

### Rust
- `tauri` 2.1 - Desktop app framework
- `tauri-plugin-dialog` 2.0 - File dialog support
- `serde` - Serialization
- `chrono` - Date/time handling
- `regex` - Pattern matching
- `dirs` - Cross-platform paths

### TypeScript
- `react` 18 - UI framework
- `@tauri-apps/api` - Tauri APIs
- `@tauri-apps/plugin-dialog` - File dialog support
- `i18next` - i18n framework
- `react-i18next` - React i18n bindings
- `i18next-browser-languagedetector` - Language detection
- `date-fns` - Date formatting
- `lucide-react` - Icons
- `@tanstack/react-virtual` - Virtual scrolling

## Status

**Version**: 0.1.0
**Phase**: Phase 2 - Feature Complete

Core MVP and Phase 2 features are implemented. The application is ready for testing and further development.

## License

MIT