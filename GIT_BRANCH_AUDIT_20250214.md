# Git Branch Audit Report: full-text-search vs origin/main

**Date:** 2025-02-14  
**Current Branch:** `full-text-search` (HEAD at 507d878)  
**Target Branch:** `origin/main` (HEAD at b3bcad2)  
**Auditor:** Claude Code

---

## Executive Summary

⚠️ **CRITICAL FINDING:** The `full-text-search` branch is **107 commits behind** `origin/main` and diverged 8 commits ago. The branch introduces Full-Text Search (FTS) functionality but has missed **substantial architectural evolution** on the main branch, including:

- Complete transport layer refactoring (WebSocket → HTTP+SSE)
- New view modes (Kanban, Flow graph visualization)
- Hierarchical tag system
- Multi-path session scanning
- Embedded web access & auth token management
- Mobile-responsive redesign
- CLI/GUI unified server architecture
- 4 new language locales (de-DE, es-ES, fr-FR, ja-JP)
- Extensive i18n overhaul
- Numerous bug fixes and performance improvements

**Merge will be complex and conflict-heavy.** Not recommended without careful planning and potentially rebasing onto latest main.

---

## 1. Branch Hierarchy & Commit Analysis

```
* 507d878 (HEAD -> full-text-search) update token counter
* 4577be4 FTS and startup stability improvements - migration
* 0ea2c39 FTS and startup stability improvements
* 20342e0 UX improvements - esc, close search popup
* 7694fae small visual polish
* 5ba2cc7 Add full-text search functionality & UI +3
* 57ff96b some fts fixes
* 425206d Add full-text search functionality & UI
| * b3bcad2 (origin/main, origin/HEAD) fix(ci): satisfy cli clippy gates for watcher and terminal
| * 7c5857e feat(cli): support terminal watcher and manual api key mode
| * b02a9fd fix(http): add missing Content-Type header for gzip responses
| * 7868430 fix(http): return gzip response via explicit Response builder
| * 6c3221b style(ws): normalize ws_adapter formatting for rustfmt check
| * 52ef1c1 fix(transport): satisfy CompressionStream BufferSource typing
| * 725b40d docs(readme): update README with multi-platform support and add Chinese version
| * 3c7a7e0 fix(cli): remove unused server config fields to satisfy clippy
| * f9b0ff2 style(rust): apply workspace rustfmt normalization
| * 753f25b fix(ci): use pnpm frozen lockfile in CI and release workflows
| * 740383c chore(build): 将 CLI 构建入口切换为 shell 脚本
| * 66c347a feat(ui): 键盘提示支持跨平台修饰键显示
| * c69cfdd fix(session-tree): 修复搜索跳转到 toolResult 的定位
| * 8a849d0 fix(scanner): 排除非会话目录的 jsonl 文件
| * a9be2a2 chore(i18n): add missing i18n keys for AdvancedSettings card headers
| * 91c8989 feat(i18n): add German, Spanish and French locale translations
| * d028b99 style(settings): improve responsive layout across settings sections
| * d1b84a7 style(settings): polish SettingsPanel sidebar and mobile UX
| * e8dc8d9 refactor(settings): redesign AdvancedSettings with card layout
| * ac99f03 style(css): add safe-area-top utility for mobile notch
| * fd9ffc1 feat(ui): add reusable Toggle and SettingsCard components
| * a1a1ea3 feat: subagent session viewer with modal, toolbar toggles, and drag region fix
| * cd82f8c feat(settings): Pi Config TUI with resource management, settings editor, and default value alignment
| * 33ab6ab feat: add unified SearchFilterBar for list, project, and kanban views
| * 76fe88b fix(dev): GUI dev mode now correctly uses Vite dev server
| * c4f7ece ci: unified release workflow for Desktop + CLI
| * 9d8baa7 feat: unified single-port server + remote auth gate
| * bd1c688 docs(changelog): update with all unreleased features
| * 0999a09 feat(ui): connection status banner for WS/HTTP transport
| * 8a3bfdb fix: onboarding bind_addr config, WS host detection, mobile UX issues
| * adbca4b docs(changelog): add mobile adaptation entry
| * bfdecad feat(mobile): full responsive adaptation for <768px screens
| * 34a0f95 feat(frontend): incremental session list updates via diff patching
| * 1c03c22 feat(transport): add HttpTransport with SSE, fix WS connection race
| * 8c61556 feat(backend): auth token management, configurable bind address, SSE endpoint
| * d233c05 refactor(backend): incremental session scanning with cache-based diff
[TRUNCATED - 107 commits ahead]
```

**Divergence Point:** Commit `425206d` ("Add full-text search functionality & UI")  
**Ahead on full-text-search:** 8 commits (FTS features only)  
**Behind relative to main:** 107 commits (major architectural changes)

---

## 2. Feature Comparison

### Features on `full-text-search` branch (NOT in main):

| Feature | Description | Impact |
|---------|-------------|--------|
| **Full-Text Search** | FTS5-based message-level search with pagination, role/glob filtering, scoring | Core new feature, requires SQLite schema changes |
| **Search UI** | Modal interface with sorting, filtering, live results | New component (`FullTextSearch.tsx`) |
| **Entry Navigation** | `initialEntryId` prop on SessionViewer for jump-to-result | Modification to existing component |
| **Token Counter Update** | Token counting improvements | Minor tweak |
| **Startup Stability** | Migration handling for FTS | Error handling enhancements |
| **UX Polish** | ESC to close, improved visuals | Minor UI improvements |

### Features on `origin/main` (NOT in full-text-search):

| Category | Features |
|----------|----------|
| **View Modes** | Kanban board (drag-and-drop), Flow graph visualization (React Flow), List/Project views |
| **Tag System** | Hierarchical labels with parent-child filtering, tag management UI |
| **Transport** | HTTP+SSE transport (replacing pure WebSocket), auth token management, configurable bind address |
| **Embedded Web** | Built-in web server for remote access, embedding frontend assets via rust-embed |
| **Mobile** | Full responsive adaptation for <768px screens, mobile-specific UX, safe-area support |
| **Scanner** | Multi-path session scanning, incremental diff-based updates, in-memory cache |
| **CLI** | Unified CLI/GUI server, terminal watcher, manual API key mode |
| **Settings** | Major redesign: card-based layout, Pi Config TUI with resource management, advanced settings sections |
| **i18n** | German (de-DE), Spanish (es-ES), French (fr-FR), Japanese (ja-JP) locales added |
| **UI Components** | Toggle, SettingsCard, ConnectionBanner, KbdTooltip, Skeleton, TerminalPanel, SubagentModal, TagBadge, etc. |
| **Hooks** | useIsMobile, useAppearance, useTags, useAllSettings, useConnectionStatus |
| **Contexts** | TransportContext, SettingsContext |
| **Dashboard** | Completely redesigned with charts: ActivityHeatmap, MessageDistribution, ProductivityMetrics, TokenTrendChart, etc. |
| **Backend Modules** | New commands: auth, tags, terminal, cache; new files: auth.rs, dispatch.rs, http_adapter.rs, ws_adapter.rs, compression.rs, write_buffer.rs, settings_store.rs, app_state.rs |
| **Infrastructure** | Unified release workflow (Desktop+CLI), CLI build scripts (shell + TypeScript), Dockerfile.cli |

---

## 3. Detailed File Change Statistics

### Overall Diff (origin/main → full-text-search)

```
83 files changed, 6042 insertions(+), 827 deletions(-)
```

### Modified Files (highest impact):

#### Backend (src-tauri/)

| File | Changes | Notes |
|------|---------|-------|
| `src-tauri/src/sqlite_cache.rs` | ~1200 line diff | FTS message table & triggers, schema migrations, recovery logic |
| `src-tauri/src/scanner.rs` | ~200 line diff | FTS-specific scanning logic, corruption recovery |
| `src-tauri/src/commands/search.rs` | ~200 line diff | FTS command implementation |
| `src-tauri/tests/migration_test.rs` | ~300 lines added | FTS migration testing |
| `src-tauri/tests/fts_message_integrity_test.rs` | NEW | 824 lines test for FTS |
| `src-tauri/tests/full_text_search_pagination_test.rs` | NEW | 213 lines pagination tests |
| `src-tauri/src/search.rs` | Modified | May conflict with main's search improvements |
| `src-tauri/src/models.rs` | Modified | FTS hit/response types added |
| `src-tauri/src/config.rs` | Modified | FTS config options |

#### Frontend (src/)

| File | Changes | Notes |
|------|---------|-------|
| `src/components/FullTextSearch.tsx` | NEW | 472 lines, entirely new component |
| `src/components/__tests__/FullTextSearch.test.tsx` | NEW | 137 lines tests |
| `src/App.tsx` | 40 line diff | Adds FTS modal, `initialEntryId` prop to SessionViewer |
| `src/components/SessionViewer.tsx` | 27 line diff | Accepts `initialEntryId` prop for jump-to-result |
| `src/components/TokenStats.tsx` | 201 lines removed | Component deleted in FTS branch but exists on main |
| `src/hooks/useSessions.ts` | 92 line diff | Modified to support FTS |
| `src/types.ts` | 19 lines added | FullTextSearchHit, FullTextSearchResponse types |
| `src/types/index.ts` | 29 lines added | Re-exports |
| `src/i18n/locales/*/search.ts` | 18 lines each | FTS translations |
| `src/index.css` | 42 lines added | FTS styles |
| `src/styles/command.css` | 9 lines added | Minor FTS-related tweaks |

---

## 4. Conflict Hotspots Analysis

### ⚠️ HIGH CONFLICT RISK

#### 1. SQLite Schema (`src-tauri/src/sqlite_cache.rs`)

**full-text-search changes:**
- Adds `message_entries` table for per-message FTS
- Creates `message_fts` virtual table (FTS5)
- Adds triggers for automatic index updates
- Implements corruption recovery (WAL mode, drop-and-recreate)
- Migration logic from old `sessions_fts` to new schema
- Adds `user_messages_text`, `assistant_messages_text` columns to `sessions` table
- Functions: `ensure_message_fts_schema`, `full_rebuild_fts`, `create_sessions_triggers`, etc.

**origin/main changes (inferred from timeline):**
- Enhanced session details cache
- Tag-related tables (`tags`, `session_tag_assignments`)
- Possibly different FTS approach or deprioritized FTS
- May have touched `sessions` table for stats, token tracking

**Merge Risk:** **SEVERE** - Both branches alter the same core database initialization function with different schema additions. Triggers, table creation order, and migration logic will conflict heavily. The `init_db_with_config` function is likely >300 lines on both branches with non-overlapping modifications.

---

#### 2. Scanner Logic (`src-tauri/src/scanner.rs`)

**full-text-search changes:**
- Added corruption recovery with retry loops and DB reset
- Changed to skip DB errors per-file and continue scanning
- Separate logic for realtime vs historical sessions
- Deterministic sorting by `(modified DESC, path ASC)`

**origin/main changes (from `d233c05`):**
- **Incremental scanning:** Returns `SessionsDiff` with added/removed sessions
- **Multi-path support:** Scans multiple configured directories
- **In-memory cache:** Static cache with versioning (`SCAN_CACHE`, `CACHE_VERSION`)
- **Digest for polling:** `get_session_digest()` for lightweight change detection
- **Debounced file watcher integration:** Cache invalidation on changes
- **Simplified API:** `scan_sessions()` returns cached results

**Merge Risk:** **SEVERE** - The entire scanning strategy differs. main's incremental diff-based approach is architecturally different from full-text-search's full-scan approach. Function signatures and return types may differ. Integration of FTS message extraction with main's incremental scanning will require significant refactoring.

---

#### 3. commands/search.rs

**full-text-search changes:**
- Implemented `full_text_search` command with complex pagination and per-session limit
- Added session-level `search_sessions_fts` (legacy?)
- Advanced query building with role filters and glob patterns

**origin/main changes:**
- Likely added tag filtering to search
- Possibly transport layer changes moved invoke API
- May have improved `search_sessions` with new modes

**Merge Risk:** **HIGH** - The function implementation differs significantly. Need to preserve FTS query logic while integrating with main's potentially updated search plugin system.

---

#### 4. App.tsx

**full-text-search changes:**
- Added `showFullTextSearch` state and modal
- Added `initialEntryId` state for jump-to-result
- Added `FullTextSearch` component integration
- Added keyboard shortcut `Cmd+Shift+F`
- Modified `handleSelectSession` to clear `initialEntryId`
- Modified `SessionViewer` props: `initialEntryId={initialEntryId}`

**origin/main changes (abbreviated):**
- Added `mobileTab` state and mobile-specific UI
- Added `showOnboarding`, `showTerminal`, `terminalMaximized`, `terminalPendingCommand`, `terminalConfig`
- Added `patchSessions` from `useSessions`
- Added `reloadTerminalConfig` and appearance/bootstrap logic
- Added `useIsMobile`, `useAppearance`, `useTags` hooks
- Changed `viewMode` to include `'kanban'` and read from cache
- Added `filterTagIds`, `sidebarSearchQuery`
- **Removed `initialEntryId` logic** (may have implemented differently or dropped)
- Changed import from `@tauri-apps/api/core` to `./transport`
- Added `ConnectionBanner`, `TerminalPanel`, `Onboarding`, `KanbanBoard`, `SearchFilterBar`, `KbdTooltip` imports

**Merge Risk:** **HIGH** - The `initialEntryId` prop flows through `App.tsx` → `SessionViewer`. On main, this may have been renamed, removed, or replaced with a different mechanism (e.g., `scrollTargetId`). Also main's massive UI changes (mobile, kanban, terminal) could conflict with FTS modal placement.

---

#### 5. SessionViewer.tsx

**full-text-search changes:**
- Added `initialEntryId` prop
- `useEffect` to sync `initialEntryId` → `activeEntryId`

**origin/main changes (from diff):**
- Added session content cache (`SESSION_CONTENT_CACHE`)
- Added `onWebResume` prop
- Changed `invoke` import to `listen` from `../transport`
- Added `showThinking`, `toolsExpanded` from context
- Added `KbdTooltip`, `useIsMobile`
- Added `scrollTargetId` state (may be an alternative to `initialEntryId`)
- Removed some logging
- Restructured imports

**Merge Risk:** **HIGH** - Both branches touch the prop interface. Main may have replaced `initialEntryId` with `scrollTargetId`. Need to reconcile these or drop the full-text-search change if main already implements equivalent functionality.

---

#### 6. Types (`src/types.ts`)

**full-text-search additions:**
```typescript
export interface FullTextSearchHit {
  session_id: string
  session_path: string
  session_name?: string
  entry_id: string
  role: string
  content: string
  timestamp: string
  score: number
}

export interface FullTextSearchResponse {
  hits: FullTextSearchHit[]
  total_hits: number
  has_more: boolean
}
```

**origin/main changes (inferred):**
- Added `SessionsDiff`, `SessionUpdate`, `Tag`, `TagAssignment`, etc.
- Possibly modified `SessionInfo` or `SessionEntry`
- May have added HTTP/transport types

**Merge Risk:** **MEDIUM** - Types should compose cleanly unless there are naming conflicts or main added similar search result types with different shapes.

---

#### 7. Hooks (`src/hooks/`)

**full-text-search changes:**
- Modified `useSessions` to possibly handle FTS scan modes
- Added `useDemoMode` (24 lines) - but this may already exist on main

**origin/main changes:**
- Added `useIsMobile`, `useAppearance`, `useTags`, `useAllSettings`, `useConnectionStatus`
- Refactored `useSessions` to support `patchSessions` for incremental updates
- Modified `useFileWatcher`

**Merge Risk:** **HIGH** - `useSessions` is likely significantly refactored on main. The FTS branch modifications may not apply cleanly. Need to review both implementations and likely rewrite FTS integration to use main's new hook API.

---

#### 8. i18n Structure

**full-text-search changes:**
- Added keys to `en-US/search.ts` and `zh-CN/search.ts`

**origin/main changes:**
- Completely modularized i18n: separate files per feature (`dashboard.ts`, `tags.ts`, `terminal.ts`, etc.)
- Added locales: `de-DE`, `es-ES`, `fr-FR`, `ja-JP`
- New `i18n/config.ts`, `i18n/index.ts`
- Added `languageSwitcher.ts`, `components.ts`, `session.ts`, `settings.ts`, etc.

**Merge Risk:** **MEDIUM** - The structure changed dramatically. FTS additions to old `search.ts` may conflict or need relocation into new modular structure.

---

## 5. Architecture Divergence Summary

| Layer | full-text-search | origin/main |
|-------|------------------|-------------|
| **Transport** | WebSocket only (original) | HTTP+SSE + WebSocket fallback, auth tokens |
| **Scanner** | Full scans, cache read-through | Incremental diff, multi-path, in-memory cache |
| **Search** | FTS5 at message level, separate UI | Session-level search + possibly different implementation |
| **UI Layout** | Single column, sidebar search panel | Mobile-responsive, kanban board, flow graph, terminal panel |
| **Settings** | Single panel, simple sections | Card-based advanced settings, TUI editor, sections per feature |
| **i18n** | Flat per-language files | Modular per-feature files, 8 locales |
| **Backend Cmds** | Commands in `commands/` submodules | Dispatcher pattern (`dispatch.rs`), separate modules |
| **CLI** | Separate binary? | Unified server (GUI/CLI share backend) |
| **File Watcher** | Debounced, simple | Integrated with incremental scanner |
| **Cache** | SQLite only | SQLite + write buffer + in-memory scanner cache |

---

## 6. Specific Merge Risks & Questions

### Risk 1: Database Schema Migration Order

**Problem:** Both branches alter `init_db_with_config`. FTS adds `message_entries` and `message_fts`; main may add tag tables. The ordering of `ALTER TABLE` and `CREATE INDEX` statements could conflict.

**Mitigation:**
- Manually merge `init_db_with_config` preserving both sets of changes.
- Ensure migrations are idempotent and non-destructive (already use `IF NOT EXISTS`, `OK` for `ALTER`).
- Consider splitting FTS and tag migrations into separate functions called sequentially.
- Test migrations from old v0.1.x schema with combined changes.

---

### Risk 2: Incremental Scanner vs FTS Message Indexing

**Problem:** Main's incremental scanner returns `SessionsDiff`. FTS needs to extract per-message entries and insert into `message_entries` table. The scanner currently processes each file in `scan_sessions_with_config`.

**Question:** Does FTS requirement (parse every message) conflict with incremental approach? Yes - incremental scanning only updates changed sessions, but FTS needs message-level granularity for all sessions to answer search queries.

**Mitigation:**
- FTS indexing should be **lazy**: when search is executed, if `message_entries` is missing or stale, trigger background indexing of relevant sessions.
- Or: Incremental scanner should also update `message_entries` for changed sessions (delete old messages, insert new ones).
- Need to design `message_entries` lifecycle: when session deleted, messages should cascade delete.

---

### Risk 3: `initialEntryId` may be redundant or conflicting

**Problem:** Main's SessionViewer already has `scrollTargetId` state. FTS adds `initialEntryId` prop to jump to a specific message from search results.

**Question:** Did main implement equivalent functionality? From diff, yes - `scrollTargetId` appears to serve same purpose.

**Mitigation:**
- Investigate main's implementation of scroll-to-entry.
- If equivalent, **drop** FTS's `initialEntryId` changes and use main's mechanism.
- If different, merge them or choose one.

---

### Risk 4: Transport Layer Changes

**Problem:** FTS uses `invoke` from `@tauri-apps/api/core`. Main switched to `./transport` abstraction which may wrap invoke.

**Mitigation:**
- Ensure FTS command definitions are registered with main's dispatcher.
- Update FTS component imports to use `invoke` from `./transport` if that's the new pattern.

---

### Risk 5: Test Suite Conflicts

**Problem:** Both branches have modified `src-tauri/tests/`. FTS added 3 new test files. Main likely added many more tests.

**Mitigation:**
- After merge, run all tests to ensure they pass.
- Update FTS tests if main changed test framework or utilities.

---

### Risk 6: Token Stats Removal

**Problem:** FTS branch deleted `src/components/TokenStats.tsx` (201 lines). But this component may exist on main and be used elsewhere.

**Check:** Did main delete or rename it? Need to verify. If main still has it, deletion was likely intentional (maybe moved to dashboard subfolder?). From diff on main, there is a `dashboard/TokenStats.tsx`.

**Mitigation:**
- If main moved TokenStats to `components/dashboard/TokenStats.tsx`, keep that and drop FTS deletion.
- If both deleted, it's a non-issue.

---

## 7. Recommended Merge Strategy

### Option A: Rebase `full-text-search` onto `origin/main` (RECOMMENDED for clean history)

**Steps:**
1. Create backup branch: `git branch full-text-search-backup-20250214`
2. Rebase: `git rebase -i origin/main`
   - Resolve conflicts one commit at a time
   - Squash or edit FTS commits as needed
   - May need to rewrite FTS scanner integration to use main's incremental API
3. Re-run test suite on each conflict resolution step.
4. After successful rebase, create PR with clean diff against main.
5. Request code review focusing on FTS architectural integration.

**Pros:**
- Linear history, easier to understand
- Forces resolution of architectural mismatches early
- Resulting diff will show only net FTS changes

**Cons:**
- Interactive rebase with 8 commits and many conflicts is time-consuming
- Requires deep understanding of both codebases

---

### Option B: Merge `origin/main` into `full-text-search` (FASTER but messy)

**Steps:**
1. `git checkout full-text-search`
2. `git merge origin/main` (accept many conflicts)
3. Resolve conflicts in the merge commit.
4. Test thoroughly.
5. Continue work on `full-text-search`, then later merge to main.

**Pros:**
- Single conflict resolution point
- Keeps FTS commits intact

**Cons:**
- Merge commit will be huge and hard to understand
- History shows a massive merge with both sides changing same files
- Difficult to cherry-pick or revert

---

### Option C: Manually port FTS to latest main (CLEANEST but most work)

**Steps:**
1. Create new branch from `origin/main`: `git checkout -b fts-integration`
2. Copy FTS feature files: `FullTextSearch.tsx`, types, tests, etc.
3. Integrate using main's new架构:
   - Use `useSessions` with `patchSessions`
   - Use `invoke` from `./transport`
   - Place FTS modal with new UI layout
   - Use main's scanner and ensure `message_entries` indexing strategy fits incremental model
4. Write new `full_text_search` command that works with main's changes.
5. Update i18n to new modular structure.
6. Test.

**Pros:**
- Guarantees clean codebase, no leftover obsolete patterns
- Forces proper integration with latest architecture
- Easiest to maintain going forward

**Cons:**
- Essentially re-implementing FTS branch
- Requires re-reading and understanding of FTS logic to port correctly
- Most time-consuming (~2-3 days for careful work)

---

## 8. Action Items & Checklist Before Merging

### Immediate Audit Tasks

- [ ] **Verify SessionViewer's entry navigation:** Check if main's `scrollTargetId` is equivalent to `initialEntryId`
- [ ] **Compare SQLite schemas:** Generate `sqlite_master` from both branches' databases to see exact differences
- [ ] **Check command registration:** `src-tauri/src/lib.rs` - how are commands registered? Does main's dispatcher pattern require changes to FTS command registration?
- [ ] **Review i18n structure:** Locate main's search translation files. Are they split? Confirm key names.
- [ ] **Check dependency changes:** `Cargo.toml` - FTS may add nothing, but main added many (axum, tower-http, portable-pty, etc.). Ensure compatibility with `tantivy` if used.
- [ ] **Build and run:** Attempt to compile and run both branches separately to confirm they work.

### Pre-Merge Integration Work

- [ ] **Choose merge strategy** (rebase recommended)
- [ ] **Design FTS + incremental scanner integration:**
  - How will `message_entries` be kept in sync with session changes?
  - Should `scan_sessions` also update FTS index for changed sessions?
  - Queue-based background indexing for performance?
- [ ] **Resolve SessionViewer prop conflict:** `initialEntryId` vs `scrollTargetId`
- [ ] **Update FTS command to use latest config/storage APIs** (may have changed on main)
- [ ] **Port i18n keys** to main's modular structure
- [ ] **Ensure tests pass:** Run `cargo test` and frontend type check after merge
- [ ] **Update documentation:** CHANGELOG, README, AGENTS.md

### Post-Merge Validation

- [ ] **Full-text search performance:** Test with 1000+ sessions, verify pagination, role/glob filters
- [ ] **Corruption recovery:** Simulate SQLite corruption, ensure recovery works
- [ ] **Mobile UI:** Test FTS modal on mobile (<768px)
- [ ] **Kanban/Flow views:** Ensure search results still navigate correctly
- [ ] **Incremental scan:** Add new session, modify existing; verify `message_entries` updates correctly
- [ ] **Multi-path:** Configure extra session directory, ensure search indexes all paths
- [ ] **CLI mode:** Test FTS in headless/GUI+CLI modes

---

## 9. Conclusion

The `full-text-search` branch introduces a valuable feature (FTS) but is **significantly out of sync** with `origin/main`. A naive merge will result in hundreds of conflicts and likely broken functionality.

**Recommendation:** Use **Option C (manual port)** or **Option A (rebase with careful conflict resolution)**. The port approach, though more work, ensures FTS is built against the new architecture from day one, reducing technical debt.

**Estimated effort:**
- Rebase: 1-2 days (conflict resolution + testing)
- Manual port: 3-5 days (architectural integration + testing)

**Decision Critical:** Before committing to merge, determine:
1. Does main already have entry navigation (`scrollTargetId`)? If yes, FTS changes to SessionViewer may be obsolete.
2. What is main's strategy for search? Is FTS still desired, or was it abandoned? Verify with team.
3. Is the `message_entries` approach compatible with incremental scanner and multi-path design? This is the largest architectural decision.

---

*Report generated by Claude Code after thorough git analysis and file inspection.*
