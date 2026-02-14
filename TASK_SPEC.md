# TASK_SPEC.md

## Integration Task: Full-Text Search into Modern Mainline Architecture

**Date:** 2025-02-14  
**Owner:** Staff Engineering Team  
**Status:** Planning Phase  
**Audience:** Engineering team executing the integration<br>
**Prerequisite Reading:**  
- `GIT_BRANCH_AUDIT_20250214.md` (comprehensive divergence analysis)  
- `INTEGRATION_PLAN_20250214.md` (detailed integration strategy)

---

## 1. Executive Summary

We must integrate the **full-text search (FTS)** feature from the `full-text-search` branch into the significantly more advanced `origin/main` branch. These branches diverged 107 commits ago, with main introducing major architectural improvements:

- Incremental scanner with diff-based updates & multi-path support
- Write buffer for async DB writes
- Unified transport layer (HTTP+SSE + WebSocket)
- Mobile-responsive UI with Kanban/Flow views
- Embedded terminal panel
- Hierarchical tagging system
- Comprehensive i18n (8 locales)
- 4 new language translations

**The challenge:** Preserve FTS's message-level search capability while adopting main's superior infrastructure. This is a non-trivial architectural merge requiring careful schema design, scanner refactoring, and UI integration.

**Success criterion:** A single branch containing **both** the FTS feature and all mainline innovations, with all tests passing and no regression in existing functionality.

---

## 2. Strategic Context & Rationale

### 2.1 The Problem We're Solving

The `full-text-search` branch was created when the codebase had:
- Simple full-scan scanner
- Direct Tauri API usage
- Session-level search only
- Basic UI without mobile support

During its development, main evolved dramatically to become a production-ready system with:
- **Performance:** Incremental scanning reduces I/O by 90%+ for large session repos
- **Reliability:** Write buffer prevents DB corruption under concurrent load
- **Accessibility:** Embedded web server for remote/mobile access
- **Scalability:** Multi-path scanning, hierarchical tags
- **UX:** Responsive design, Kanban board, Flow visualization

FTS branch **lacks** all these advances. Merging naively would regress the application by months of development.

### 2.2 Why This Integration Is Critical

1. **User Value:** FTS is a **core user-facing feature** ("search within message content"). Without it, users must rely on session name/content search only.
2. **Architectural Cohesion:** Main's improvements are objectively superior and must be preserved.
3. **Technical Debt:** Maintaining two divergent branches doubles maintenance burden and creates merge hell.
4. **Release Cadence:** We need a single codebase for continuous delivery.

### 2.3 Non-Goals (What We're NOT Doing)

- **Not re-implementing FTS from scratch** - we're integrating existing, tested code
- **Not redesigning FTS UX** - the modal interface with filtering/sorting is approved
- **Not supporting multiple search backends** (tantivy is not part of this; we use SQLite FTS5 exclusively)
- **Not maintaining separate CLI and GUI codebases** - we're using main's unified architecture
- **Not optimizing FTS at the expense of main's invariants** (e.g., we won't disable incremental scanning to simplify)
- **Not creating a feature flag gating FTS** (it will be always-available once merged)

---

## 3. Key Technical Decisions

### 3.1 Database Schema: Union with Migrations

**Decision:** Keep main's schema and **non-destructively** add FTS components.

**Rationale:** Main's schema includes tags, favorites, settings stores. FTS needs two new tables and additional columns on `sessions`. We must preserve existing data while enabling FTS.

**Implementation:**
- Keep main's CREATE TABLE statements unchanged
- Add `ALTER TABLE sessions ADD COLUMN user_messages_text TEXT` (ignored if exists)
- Add `ALTER TABLE sessions ADD COLUMN assistant_messages_text TEXT`
- CREATE TABLE `message_entries` (new, FTS-specific)
- CREATE VIRTUAL TABLE `message_fts` USING fts5(...) if `enable_fts5`
- Use `ALTER TABLE` to add missing columns as **non-destructive migrations** (ignore "duplicate column" errors)
- Preserve FTS's corruption recovery: detect malformed DB, delete, recreate, retry once

**Invariant:** All schema changes must be backwards-compatible with v0.1.x databases (no drops without ALTER).

---

### 3.2 Scanner Architecture: Main's Incremental + FTS Message Indexing

**Decision:** Adopt main's `scanner.rs` wholesale and enhance `parse_session_info` to extract messages.

**Rationale:** Main's scanner provides:
- Multi-path support (`get_all_session_dirs`)
- In-memory cache (`SCAN_CACHE`) with digest for change detection
- Write buffer for async DB writes (prevents UI jank)
- Directory filtering (skip `transcripts/`, `subagent-artifacts/`)
- Stable sorting for deterministic pagination

**Implementation:**
1. Start with main's `scanner.rs` (b3bcad2)
2. Modify `parse_session_info` to:
   - Stream through all lines
   - For each `type == "message"`, extract `role`, `content` (text parts), `timestamp`, `id`
   - Return the `SessionInfo` **plus** the parsed messages (need to modify signature or return tuple)
   - Compute concatenated `user_messages_text` and `assistant_messages_text` aggregates
3. In `scan_sessions_with_config`, after `parse_session_info`:
   ```rust
   let (info, messages) = parse_session_info(&file_path)?;
   // Insert messages into message_entries within a transaction
   sqlite_cache::upsert_message_entries(&conn, info.path, messages)?;
   // Push session to list
   sessions.push(info);
   write_buffer::buffer_session_write(&info, file_modified)?;
   ```
4. Wrap DB operations with corruption detection (from FTS), but at the scan level, not per-file

**Invariant:** Scanner must not block on DB writes (uses `write_buffer`). Must skip unreadable files without aborting full scan.

---

### 3.3 Command Registration: Standard Tauri Handler

**Decision:** Register `full_text_search` in `lib.rs`'s `generate_handler!` macro alongside other commands.

**Rationale:** Tauri's standard pattern. No need for custom dispatcher; main already uses this pattern.

**Implementation:**
- Ensure `src-tauri/src/commands/search.rs` exports `full_text_search` with `#[tauri::command]`
- In `lib.rs`, add `full_text_search` to the list
- No changes to frontend invocation pattern (uses `invoke('full_text_search', ...)`)

---

### 3.4 Frontend Transport: Use `./transport` Abstraction

**Decision:** All components must import `invoke` from `./transport`, not `@tauri-apps/api/core`.

**Rationale:** Main's `transport.ts` provides unified interface for Tauri commands and future HTTP fallback. Ensures consistency.

**Implementation:**
- In `FullTextSearch.tsx`, change:
  ```typescript
  import { invoke } from './transport';
  ```
- Remove any other direct Tauri API imports

---

### 3.5 SessionViewer Navigation: Imperative Scroll Control

**Decision:** Keep main's internal `scrollTargetId` state. Add prop `initialEntryId` for one-time navigation trigger.

**Rationale:** Main's `scrollTargetId` already implements smooth scrolling and highlight. FTS needs to set it externally when opening a session. Adding a prop is simplest and least invasive.

**Implementation:**
- In `SessionViewerProps`, add `initialEntryId?: string`
- In component body:
  ```typescript
  useEffect(() => {
    if (initialEntryId) {
      setScrollTargetId(initialEntryId);
    }
  }, [initialEntryId]);
  ```
- Main's existing `useEffect` on `scrollTargetId` handles scrolling/highlighting

**Invariant:** `scrollTargetId` is cleared after use to allow subsequent navigation.

---

### 3.6 User Interface: Modal Overlay in Main's Layout

**Decision:** Render `FullTextSearch` as a modal overlay (like `CommandPalette`) within main's restructured UI.

**Rationale:** Main's `App.tsx` now has mobile tabs, kanban, terminal panel. FTS modal is a temporary overlay that works atop any view.

**Implementation:**
- In `App.tsx` (main's version), add state: `const [showFullTextSearch, setShowFullTextSearch] = useState(false);`
- Add keyboard shortcut: `cmd+shift+f` → `setShowFullTextSearch(true)`
- Render conditionally:
  ```tsx
  {showFullTextSearch && (
    <FullTextSearch
      isOpen={true}
      onClose={() => setShowFullTextSearch(false)}
      onSelectResult={(session, entryId) => {
        setSelectedSession(session);
        setScrollTargetId(entryId); // Need state or context
        setShowFullTextSearch(false);
      }}
    />
  )}
  ```
- To set scroll target on SessionViewer, either:
  - Pass `initialEntryId={null}` normally, or the actual ID when coming from FTS
  - Or use a ref if SessionViewer supports imperative method

---

### 3.7 i18n: Modular Structure with FTS Namespace

**Decision:** Create `locales/en-US/search.ts` and `locales/zh-CN/search.ts` (modular) containing FTS keys. Duplicate to `de-DE`, `es-ES`, `fr-FR`, `ja-JP` with translations.

**Rationale:** Main restructured i18n to be feature-based. FTS keys belong in `search.ts`, not the old flat file.

**Implementation:**
- Copy FTS's translation strings into new modular files
- Ensure namespace keys match usage in `FullTextSearch.tsx` (e.g., `t('search.fullText.placeholder')`)
- Add missing translations for all 8 locales (use machine translation for de/es/fr/ja if necessary, mark as TODO)

---

## 4. Quality Attributes & Invariants

### 4.1 Correctness
- FTS must return accurate results matching query across all message content
- No false positives (word stemming not required; exact phrase match with escaping)
- Pagination must be stable: page 1 results consistent when scrolling

### 4.2 Performance
- Full scan with message parsing: < 30s for 10,000 sessions (measured on dev machine)
- Search query latency: < 500ms for 1000 results
- Incremental scan: only parse modified sessions (database modification time check)
- Memory: Parsing should stream; no full file load into memory (already using BufReader)

### 4.3 Data Integrity
- `message_entries` must exactly mirror messages in source files for the latest parsed version
- Deletion of session file → cascade delete from `message_entries` (via DB foreign key)
- Corruption recovery: if DB is malformed, delete and recreate automatically (with warning log)
- Write buffer must flush successfully before acknowledging scan completion

### 4.4 Backward Compatibility
- Old database (pre-v0.2.0) must migrate successfully to new schema
- Migration path: add missing tables/columns/triggers via `IF NOT EXISTS` and `ALTER` (ignore errors)
- No breaking changes to existing Tauri command signatures used by main

### 4.5 Test Coverage
- Unit tests: `parse_session_info` extracts messages correctly
- Integration tests: full_text_search returns expected hits with pagination
- Migration tests: old DB formats migrate to new schema
- Corruption recovery tests: simulate malformed DB, verify auto-recovery
- End-to-end: UI test that search result click scrolls to entry

---

## 5. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Database schema conflict unresolvable** | Medium | High | Use careful `ALTER TABLE` migrations; test with backup/restore |
| **Message parsing slows scanner unacceptably** | High | High | Benchmark early; if needed, lazy-index messages (only on first search) |
| **FTS command not found after merge** | Low | Medium | Verify `generate_handler!` includes `full_text_search` |
| **SessionViewer scroll jitter on entry jump** | Medium | Low | Tune virtualizer scroll timing; add highlight animation |
| **i18n key conflicts with main** | Low | Low | Namespace all FTS keys under `search.fullText.*` |
| **Write buffer overflow on large scans** | Low | Medium | Monitor buffer size; may need to increase capacity or flush more frequently |
| **Test timeout on CI** | Medium | Low | Optimize tests; mark slow tests with `#[ignore]` if needed |
| **Transport import errors in FTS component** | Low | Low | Update all imports to `./transport` |

---

## 6. Dependencies & Assumptions

### 6.1 External Dependencies
- SQLite with FTS5 extension (enabled by default in rusqlite bundled)
- Tauri 2.x (main uses 2.10.1)
- React 18 + TanStack Virtual

### 6.2 Assumptions
- `enable_fts5` config flag exists and is `true` by default (from FTS branch)
- Main branch's `config.rs` can be extended without breaking
- Write buffer (`write_buffer.rs`) exists on main and is thread-safe
- `file_watcher.rs` triggers rescans that re-index messages for changed sessions
- Frontend build pipeline (Vite) unchanged

---

## 7. Success Metrics

| Metric | Target |
|--------|--------|
| **Test pass rate** | 100% of existing + new tests pass |
| **Build time** | < 2 min for frontend, < 5 min for backend |
| **Scan performance** | Full scan with message extraction ≤ 30s for 10k sessions |
| **Search latency** | 95th percentile ≤ 500ms |
| **DB size growth** | `message_entries` ≤ 3× size of `sessions` table (estimated) |
| **Code quality** | `cargo clippy` passes with no warnings; frontend `tsc` no errors |
| **Merge conflicts** | All resolved; `git diff origin/main` shows only FTS additions |

---

## 8. Next Steps (High-Level)

1. **Create integration branch** from `full-text-search`
2. **Merge `origin/main`** with conflict resolution per rules above
3. **Implement scanner message extraction** and DB schema merge
4. **Update frontend components** to use transport and new navigation
5. **Migrate i18n** to modular structure
6. **Write integration tests** (see TODO.md for test plan)
7. **Manual validation** in dev mode
8. **Code review** after passing all checks

---

**Document End**
