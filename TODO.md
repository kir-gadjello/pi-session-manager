# TODO.md — Full-Text Search Integration

**Project:** Integrate FTS into modern mainline architecture  
**Branch:** `fts-integration-20250214`  
**Owner:** Engineering Team  
**Last Updated:** 2025-02-14

---

## Prioritization Legend

- **P0 (Critical):** Blocking; must complete before any other work
- **P1 (High):** Required for feature completeness; do early
- **P2 (Medium):** Important but can follow after P0/P1
- **P3 (Low):** Polish, docs, non-blocking

---

## Phase 0: Setup & Baseline

### P0-1: Create integration branch and snapshot
- [x] `git checkout full-text-search`
- [ ] `git status` (verify clean)
- [ ] `git checkout -b fts-integration-20250214`
- [ ] Run baseline tests: `cargo test --locked > test_results_fts_baseline.txt 2>&1`
- [ ] Run baseline build: `npm run build > build_frontend_baseline.txt 2>&1`
- [ ] Tag baseline: `git tag fts/pre-integration-$(date +%Y%m%d-%H%M%S)`
- [ ] Push branch to remote (optional backup)

**Acceptance:** Branch exists, baseline captured

---

## Phase 1: Initial Merge of origin/main

### P0-2: Merge main with conflict halt
- [ ] `git fetch origin main`
- [ ] `git merge origin/main --no-commit --no-ff`
- [ ] Count conflicted files: `git diff --name-only --diff-filter=U | wc -l` (expect ~100)
- [ ] Do NOT attempt to resolve yet; just capture conflict state

**Acceptance:** Merge attempted, conflicts present, commit not created

---

## Phase 2: Database Schema Unification

### P0-3: Merge `sqlite_cache.rs` (most critical)
- [ ] Open `src-tauri/src/sqlite_cache.rs` (both versions)
- [ ] Start from **main's version** as base (preserve its tables: tags, favorites, settings_store)
- [ ] Add FTS's `ALTER TABLE` for `user_messages_text` and `assistant_messages_text`
- [ ] Add `CREATE TABLE message_entries` with constraints
- [ ] Add `CREATE INDEX idx_message_entries_session`
- [ ] Add `init_message_fts()` function (from FTS) called when `config.enable_fts5`
  - Create virtual table `message_fts` with `content='message_entries'`
  - Create triggers for updates if needed (consult FTS code)
- [ ] Add `ensure_message_fts_schema()` for migrations
- [ ] Add corruption recovery wrapper in `init_db_with_config()` (FTS pattern)
- [ ] Keep main's WAL mode, synchronous, foreign_keys
- [ ] Keep FTS's `full_rebuild_fts()` for rebuilding index
- [ ] Run `cargo check` → fix compile errors

**Acceptance:** `sqlite_cache.rs` compiles, contains merged schema logic

### P1-1: Update `Config` struct to include `enable_fts5: bool`
- [ ] Open `src-tauri/src/config.rs`
- [ ] Ensure `Config` has `enable_fts5: bool` field (default true)
- [ ] Ensure `session_paths: Vec<String>` exists (from main)
- [ ] Run `cargo check`

**Acceptance:** Config compiles with FTS flag

---

## Phase 3: Scanner Integration

### P0-4: Merge `scanner.rs` (adopt main's structure)
- [ ] Replace `scanner.rs` with **main's version** (from b3bcad2)
- [ ] Verify features present: `SCAN_CACHE`, `CACHE_VERSION`, `get_all_session_dirs`, `write_buffer`
- [ ] Run `cargo check` (expect DB errors because `parse_session_info` signature may differ)

### P1-2: Modify `parse_session_info` to extract messages
- [ ] Change signature to return `(SessionInfo, Vec<SessionEntry>)` or create new function
- [ ] Inside parser: as we iterate lines, collect `type == "message"` entries
- [ ] Build `Vec<SessionEntry>` with fields: `id`, `session_path` (from session path), `role`, `content`, `timestamp`
  - Note: `content` should be raw text extracted from the message's content array
- [ ] Also compute `user_messages_text` and `assistant_messages_text` by concatenating message text parts
- [ ] Return tuple `(SessionInfo { user_messages_text, assistant_messages_text, .. }, messages)`

**Acceptance:** New `parse_session_info` compiles and returns messages

### P1-3: Update `scan_sessions_with_config` to store messages
- [ ] After `let info = parse_session_info(&file_path)?`, destructure: `let (info, messages) = ...;`
- [ ] Call `sqlite_cache::upsert_message_entries(&conn, info.path, &messages)?;`
- [ ] Push `info` to sessions list, call `write_buffer::buffer_session_write(&info, file_modified)`
- [ ] Add `sqlite_cache::upsert_message_entries` function (to be implemented next)

**Acceptance:** Scanner compiles; messages are passed to DB layer

### P1-4: Implement `upsert_message_entries` in `sqlite_cache.rs`
- [ ] Add function:
  ```rust
  pub fn upsert_message_entries(conn: &Connection, session_path: &str, entries: &[SessionEntry]) -> Result<(), String> {
      // Begin transaction
      // DELETE FROM message_entries WHERE session_path = ?
      // INSERT each entry (or REPLACE)
      // Commit
  }
  ```
- [ ] Use prepared statements for performance
- [ ] Handle errors gracefully

**Acceptance:** Function compiles and is called by scanner

### P1-5: Add corruption recovery at scan level
- [ ] Wrap the per-directory scan loop in a retry mechanism similar to FTS's (but less granular)
- [ ] On DB error containing "malformed", "disk image", "not a database":
  - Drop connection
  - Delete DB file
  - Re-init connection
  - Restart scan from beginning (or continue with already-scanned sessions)
- [ ] Log warnings appropriately

**Acceptance:** Simulated corruption (manually corrupt DB) triggers recovery without panicking

### P1-6: Test scanner performance
- [ ] Create a test dataset with 1000 sessions (use existing test fixtures)
- [ ] Time `scan_sessions()` with message extraction
- [ ] Ensure < 30s on dev machine
- If too slow: optimize message parsing, batch DB inserts, consider lazy indexing

**Acceptance:** Performance benchmark logged

---

## Phase 4: Backend Command & Model Integration

### P0-5: Merge `models.rs`
- [ ] Start with **main's version** (has `SessionsDiff`, skip_serializing)
- [ ] Add fields to `SessionInfo`: `user_messages_text: String`, `assistant_messages_text: String`, `last_message: String', `last_message_role: String`
- [ ] Add `FullTextSearchHit` and `FullTextSearchResponse` structs
- [ ] Ensure `serde` attributes correct (deserialize from DB, serialize to frontend)
- [ ] Run `cargo check`

**Acceptance:** Model types compile

### P0-6: Update `commands/search.rs`
- [ ] Keep main's `search_sessions` (may have modifications)
- [ ] Keep FTS's `full_text_search` command implementation
- [ ] Ensure both functions have `#[tauri::command]`
- [ ] Verify `full_text_search` uses `config.enable_fts5` and new query logic with pagination
- [ ] Run `cargo check`

**Acceptance:** Search commands compile

### P0-7: Register command in `lib.rs`
- [ ] Open `src-tauri/src/lib.rs` (use main's version)
- [ ] In `tauri::generate_handler!([...])`, add `full_text_search` near other search commands
- [ ] Ensure `pub use commands::*;` exports it
- [ ] Run `cargo check`

**Acceptance:** Command is discoverable by Tauri

### P2-1: Verify `enable_fts5` default and config loading
- [ ] In `config.rs`, ensure default `Config::default()` sets `enable_fts5 = true`
- [ ] Confirm config file loading (from `~/.pi/agent/config.toml`) includes this field
- [ ] If missing, provide fallback to true

**Acceptance:** FTS enabled by default

---

## Phase 5: Frontend Transport & Types

### P1-7: Create/update `transport.ts`
- [ ] Open `src/transport.ts` (use main's version)
- [ ] Ensure it exports `invoke` that wraps Tauri's API (with fallback to HTTP if needed)
- [ ] Add type definitions for FTS commands:
  ```typescript
  invoke<'full_text_search'>('full_text_search', {
    query: string,
    roleFilter: 'all' | 'user' | 'assistant',
    globPattern?: string,
    page: number,
    pageSize: number
  });
  ```
- [ ] Run `npm run build` to verify types

**Acceptance:** `invoke` works for FTS

### P1-8: Update `src/types.ts` or `src/types/index.ts`
- [ ] Add `FullTextSearchHit` and `FullTextSearchResponse` interfaces (from FTS)
- [ ] Add `SessionsDiff` interface (from main) if not present
- [ ] Ensure `timestamp` fields are `string` (ISO8601)
- [ ] Run `npm run build` to ensure no type errors

**Acceptance:** TypeScript compiles with new types

---

## Phase 6: SessionViewer Navigation Integration

### P1-9: Merge main's `SessionViewer.tsx`
- [ ] Use main's version as base (with `scrollTargetId`, content cache, `onWebResume`, etc.)
- [ ] Add to `SessionViewerProps`: `initialEntryId?: string`
- [ ] Add `useEffect`:
  ```typescript
  useEffect(() => {
    if (initialEntryId) {
      setScrollTargetId(initialEntryId);
    }
  }, [initialEntryId]);
  ```
- [ ] Keep main's existing `useEffect` that watches `scrollTargetId` and scrolls to index
- [ ] Run `npm run build` to check props consistency in `App.tsx`

**Acceptance:** SessionViewer accepts and honors `initialEntryId`

---

## Phase 7: App.tsx Integration

### P1-10: Merge main's `App.tsx` (massive conflict)
- [ ] Start with main's version (has mobile, kanban, terminal, onboarding)
- [ ] Add state: `const [showFullTextSearch, setShowFullTextSearch] = useState(false);`
- [ ] Add state for scroll target (if not already present): `const [pendingScrollEntryId, setPendingScrollEntryId] = useState<string | null>(null);`
- [ ] Add `useIsMobile`, `useAppearance`, `useTags` hooks as main does
- [ ] Add keyboard shortcut in `useKeyboardShortcuts`:
  ```typescript
  'cmd+shift+f': () => setShowFullTextSearch(true)
  ```
- [ ] Import `FullTextSearch` component: `import FullTextSearch from './components/FullTextSearch'`
- [ ] Render FullTextSearch modal conditionally at end of JSX
- [ ] In `handleSelectSession` (from search), also set `setPendingScrollEntryId(entryId)`
- [ ] When rendering `SessionViewer`, pass `initialEntryId={pendingScrollEntryId}` and clear after mount

**Acceptance:** FTS modal opens with Cmd+Shift+F, search results selectable

---

## Phase 8: FullTextSearch Component Update

### P1-11: Update `FullTextSearch.tsx` imports
- [ ] Change `import { invoke } from '@tauri-apps/api/core'` → `import { invoke } from './transport'`
- [ ] Remove any other direct Tauri imports
- [ ] Verify all invoked commands exist: `full_text_search`
- [ ] Run `npm run build`

**Acceptance:** Component type-checks and uses correct transport

### P2-2: Ensure search results show role icons correctly
- [ ] Check that `role` field from backend maps to correct `User`/`Bot` icons
- [ ] Verify `formatRelativeTime` uses i18n keys properly

---

## Phase 9: i18n Migration

### P1-12: Create modular search translation files
- [ ] Create `src/i18n/locales/en-US/search.ts` with FTS keys:
  ```typescript
  export default {
    fullText: {
      placeholder: "Full-text search...",
      noResults: "No results found",
      // ... all keys from old flat search.ts
    },
    // possibly other namespaces
  };
  ```
- [ ] Update `src/i18n/locales/en-US/index.ts` to import from `./search`
- [ ] Create `src/i18n/locales/zh-CN/search.ts` with translations
- [ ] Update `zh-CN/index.ts` similarly
- [ ] For other locales (de-DE, es-ES, fr-FR, ja-JP), create `search.ts` with translated strings (use placeholder `TODO: translate` if needed)
- [ ] Remove old flat `src/i18n/locales/*/search.ts` if they conflict
- [ ] Run app and verify FTS modal shows translated strings

**Acceptance:** i18n loads without errors; FTS labels translated

---

## Phase 10: Testing

### P0-8: Rust unit tests - migration
- [ ] Edit `src-tauri/tests/migration_test.rs` to include FTS schema
  - Old schema: sessions table (pre-main)
  - New schema: sessions with tags + message_entries + message_fts
  - Verify migration adds all new tables/columns without data loss
- [ ] Run `cargo test migration_test`

**Acceptance:** Migration test passes for combined schema

### P0-9: Rust integration tests - FTS correctness
- [ ] Ensure `src-tauri/tests/fts_message_integrity_test.rs` compiles and runs
- [ ] Ensure `src-tauri/tests/full_text_search_pagination_test.rs` passes
- [ ] Run `cargo test fts` (filter)

**Acceptance:** FTS-specific tests pass

### P0-10: Rust integration tests - existing
- [ ] Run `cargo test --locked` (full suite)
- [ ] Fix any regressions from scanner changes
- [ ] If tests fail due to `parse_session_info` signature change, update test fixtures

**Acceptance:** All Rust tests pass

### P1-13: Frontend build & type check
- [ ] `npm run build` (frontend)
- [ ] Fix any TypeScript errors (props, imports, types)
- [ ] `npm run lint` if configured

**Acceptance:** Frontend builds cleanly

### P2-3: E2E test: search → result navigation
- [ ] Manually: Start app (`npm run tauri:dev`)
- [ ] Ensure Cmd+Shift+F opens FTS modal
- [ ] Enter query, verify results appear
- [ ] Click result, verify session opens and scrolls to entry (highlight appears)
- [ ] Test role filter (user/assistant/all)
- [ ] Test glob pattern (e.g., `*.py` file filter)
- [ ] Test pagination (scroll to bottom; more results load)

**Acceptance:** End-to-end flow works

### P2-4: Performance test (benchmark)
- [ ] Generate 5000 session files in test data dir (script?)
- [ ] Time full scan: `time invoke('scan_sessions')` from frontend console
- [ ] Verify < 30s
- [ ] Time search query with 1000 hits: `< 500ms`
- [ ] Document results in `PERFORMANCE.md`

**Acceptance:** Performance targets met

---

## Phase 11: Code Quality & Polish

### P2-5: Rust formatting and linting
- [ ] `cd src-tauri && cargo fmt --all -- --check`
- [ ] `cd src-tauri && cargo clippy -- -D warnings`
- [ ] Fix any warnings or errors
- [ ] `cargo test --locked` again to ensure no regressions

**Acceptance:** `cargo fmt` and `cargo clippy` pass

### P1-14: Update CHANGELOG.md
- [ ] Add section for next release (Unreleased)
- List:
  - Full-text search (message level)
  - Multi-path scanning
  - Kanban board view
  - Embedded terminal
  - Hierarchical tags
  - Mobile-responsive UI
  - 4 new i18n locales
  - Incremental scanner with diff updates
  - Write buffer for reliability
  - Unified transport (HTTP/SSE)

**Acceptance:** CHANGELOG updated

### P1-15: Update README.md
- [ ] Document FTS feature and how to use (Cmd+Shift+F)
- [ ] Mention performance improvements (incremental scanning)
- [ ] Add note about mobile support
- [ ] Update screenshots if needed

**Acceptance:** README reflects merged capabilities

### P3-1: Update AGENTS.md (if architecture docs changed)
- [ ] Document new scanner flow: `parse_session_info` → `write_buffer` → `sqlite_cache`
- [ ] Document `message_entries` table and `message_fts` index
- [ ] Note migration strategy for combined schema
- [ ] Update command registration pattern

**Acceptance:** AGENTS.md current

---

## Phase 12: Final Validation Checklist

### P0-11: Merge conflict resolution final check
- [ ] `git status` shows no unmerged files
- [ ] `git diff --name-only origin/main...fts-integration-20250214` shows expected additions (FTS component, types, tests)
- [ ] No accidental deletions of main features (kanban, terminal, tags)

### P0-12: Full test suite pass
- [ ] `cargo test --locked` (Rust)
- [ ] `npm run build` (frontend)
- [ ] `npm run tauri:dev` manual smoke test (open, FTS, kanban, terminal, tags)

### P0-13: Database migration test (critical)
- [ ] Take a pre-main database (v0.1.0) from backup or fixture
- [ ] Place it in `~/.pi/agent/sessions.db`
- [ ] Start app; ensure it migrates without error
- [ ] Verify FTS works on migrated data

### P0-14: Corruption recovery test
- [ ] Start app, let it create DB
- [ ] Manually corrupt DB (e.g., `echo "corrupt" >> ~/.pi/agent/sessions.db`)
- [ ] Restart app; verify it logs recovery attempt and recreates DB
- [ ] Verify app still functions (empty state ok)

### P2-6: Code review preparation
- [ ] Squash merge commits (optional, but make history clean)
- [ ] Ensure commit messages follow Conventional Commits
- [ ] Prepare PR description with link to `TASK_SPEC.md` and `INTEGRATION_PLAN_20250214.md`
- [ ] Annotate any known issues or TODOs

**Acceptance:** Branch ready for PR

---

## Rollback Criteria

If any P0 task fails irrecoverably after 2 hours of effort:
1. Abandon `fts-integration-20250214`
2. Switch to **Option C** (clean-room port) per audit report
3. Create new branch `fts-clean-port`
4. Use main as base and selectively re-add FTS components using main's patterns

Document decision in `DECISION_LOG.md`.

---

## Estimated Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| 0 Setup | 0.5 day | - |
| 1 Initial Merge | 0.5 day | - |
| 2 DB Schema | 1 day | 1 |
| 3 Scanner | 1.5 days | 1,2 |
| 4 Commands & Models | 0.5 day | 2 |
| 5 Transport & Types | 0.5 day | 4 |
| 6 SessionViewer | 0.5 day | 3 |
| 7 App.tsx | 1 day | 5,6 |
| 8 FTS Component | 0.5 day | 5 |
| 9 i18n | 0.5 day | 7 |
| 10 Testing | 1 day | 3-9 |
| 11 Polish | 0.5 day | 10 |
| 12 Final Validation | 0.5 day | 11 |
| **Total** | **~7-8 days** | |

---

## Notes for Implementers

- **Document every conflict resolution** in commit messages or a `RESOLUTIONS.md` file for future reference
- **Run `cargo check` frequently** to catch compile errors early in each phase
- **Do not optimize prematurely** - get it working first, then measure and optimize
- **Preserve FTS test files** (`fts_message_integrity_test.rs`, `full_text_search_pagination_test.rs`) - they are critical for correctness
- **Backup your work** - the merge is large; consider committing partial progress with `git commit -am "WIP: Phase X"`

---

**Document End**
