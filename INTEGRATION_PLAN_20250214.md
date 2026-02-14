# Optimal Integration Plan: FTS + Main Architecture

**Objective:** Integrate the `full-text-search` feature into the superior architecture of `origin/main` while preserving all FTS functionality.

**Strategy:** Create a new integration branch from `full-text-search`, merge `origin/main` into it, and resolve conflicts by adopting main's architectural patterns while carefully porting FTS logic to fit those patterns.

**Principle:** The mainline architecture is objectively superior. Do NOT force-fit main's code into FTS's old patterns. Instead, refactor FTS components to use:
- Dispatcher pattern (for HTTP/SSE)
- Incremental scanner with diff
- Multi-path support
- Write buffer for async DB writes
- New transport layer (`./transport`)
- Modular i18n
- Proper command registration

---

## Phase 0: Validation & Setup

### Step 0.1: Snapshot current state
```bash
# Ensure we're on full-text-search with clean working tree
git checkout full-text-search
git status

# Create backup/integration branch
git checkout -b fts-integration-20250214

# Record baseline metrics
cargo test --locked > test_results_fts_baseline.txt 2>&1
npm run build > build_frontend_baseline.txt 2>&1

# Tag baseline
git tag fts/pre-integration-$(date +%Y%m%d-%H%M%S)
```

### Step 0.2: Verify both branches independently work
- Run `npm run tauri:dev` on `full-text-search` → confirm FTS modal works (Cmd+Shift+F)
- Run `npm run tauri:dev` on `origin/main` → confirm new UI (kanban, terminal) works
- Document any issues

---

## Phase 1: Merge Origin/Main with Conflict Resolution Rules

### Step 1.1: Merge origin/main
```bash
git fetch origin main
git merge origin/main --no-commit --no-ff
```

**Expect:** Massive conflicts (~100 files). Resolve systematically.

---

## Conflict Resolution Rules (Priority Order)

### Rule 1: Infrastructure Adoption (Keep Main's)
- Keep: `dispatch.rs`, `transport.ts`, modular i18n, context providers, Settings redesign, Terminal panel, Kanban/Flow views
- Delete: Old patterns from FTS that conflict (direct `@tauri-apps/api/core` in components)
- Update FTS components to use `./transport`

### Rule 2: Data Structure Merging (Union of Fields)
- `SessionInfo`: Keep main's `#[serde(skip_serializing)]` on `all_messages_text`, add FTS's `user_messages_text`, `assistant_messages_text`
- Add: `FullTextSearchHit`, `FullTextSearchResponse`, `SessionsDiff`
- Ensure serialization works bidirectionally

### Rule 3: Scanner Architecture (Main's Base + FTS Message Indexing)
- Adopt main's `scanner.rs` with: `SCAN_CACHE`, `CACHE_VERSION`, `get_all_session_dirs`, `write_buffer`
- Keep main's directory skipping (`transcripts`, `subagent-artifacts`)
- **Modify** `parse_session_info` in scanner to also:
  - Parse all messages into `message_entries` table
  - Compute `user_messages_text` and `assistant_messages_text` aggregates
  - Use transactions for batch inserts
- Add FTS's corruption recovery as a wrapper around DB operations
- Remove FTS's per-file retry loop (main's `write_buffer` handles queuing)
- Keep main's stable sorting (by modified DESC, path ASC)

### Rule 4: Database Schema (Combine Both)
Start with main's schema, then add FTS:
1. Keep main's tables: `sessions` (as is), `tags`, `session_tag_assignments`, `settings_store`, `favorites`, etc.
2. Add to `sessions` table: `user_messages_text TEXT`, `assistant_messages_text TEXT` via `ALTER TABLE` (non-destructive)
3. Create `message_entries` table with proper constraints and indexes
4. If `enable_fts5`, create `message_fts` virtual table with content='message_entries' and appropriate triggers
5. Keep main's indexes (idx_modified, idx_cwd, idx_file_modified) and add idx_message_entries_session
6. Keep FTS's WAL mode, synchronous, foreign_keys pragmas
7. Keep corruption recovery: detect errors, delete DB, recreate, retry once
8. Keep both migration paths: old schema (no tags) → new schema, plus FTS migration from sessions_fts to message_fts if needed

### Rule 5: Command Registration (Dispatcher Pattern)
- Main's `lib.rs` uses `tauri::generate_handler![...]`
- Add `full_text_search` to that list
- Ensure `commands/mod.rs` exports it
- No custom dispatcher needed; Tauri handles invocation

### Rule 6: Frontend Transport & Hooks
- All FTS components must use `invoke` imported from `./transport` (not from `@tauri-apps/api/core`)
- Keep main's `useSessions` hook with `patchSessions`
- FullTextSearch component can still use its own internal scanning logic (calls `full_text_search` command directly)
- Ensure App.tsx can open FTS modal alongside main's UI

### Rule 7: SessionViewer Navigation
- Main's SessionViewer already has `scrollTargetId` state for jumping to entries
- **Remove** FTS's `initialEntryId` prop
- **Instead:** FTS should set the `scrollTargetId` on SessionViewer via:
  - Option A: App state → prop drilling
  - Option B: React context (SessionViewerContext)
  - Option C: useImperativeHandle ref (preferred for imperative navigation)

### Rule 8: i18n Migration
- Delete old flat `locales/*/search.ts` from FTS
- Create modular `locales/en-US/search.ts` with keys from FTS
- Copy same keys to `zh-CN/search.ts` and other locales from main
- Ensure no conflict with existing keys (namespace under `search`)

### Rule 9: Component Cleanup
- FTS deletes `TokenStats.tsx` but main has `dashboard/TokenStats.tsx` → keep and use main's
- Keep FTS's `FullTextSearch.tsx` and tests
- Keep main's new components (ConnectionBanner, TerminalPanel, KanbanBoard, etc.)

---

## Phase 3: File-by-File Merge Instructions

### 3.1 Rust Backend

#### A. `src-tauri/src/lib.rs`
- Keep main's structure (mod declarations order)
- Add missing modules: `dispatch` (if used), `write_buffer`, etc.
- In `tauri::generate_handler!([...])`, add:
  ```rust
  full_text_search,
  ```
  (place near `search_sessions_fts`)

#### B. `src-tauri/src/commands/mod.rs`
```rust
pub mod search;
// ... others
pub use crate::commands::search::*;
```

#### C. `src-tauri/src/commands/search.rs`
- Keep/main's `search_sessions` (may have updates)
- Keep FTS's `full_text_search` command (critical)
- If main removed `search_sessions_fts`, keep it (it's used by FTS? Possibly legacy; still safe)
- Ensure all functions have `#[tauri::command]` and return `Result<..., String>`

#### D. `src-tauri/src/models.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub path: String,
    pub id: String,
    pub cwd: String,
    pub name: Option<String>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub message_count: usize,
    pub first_message: String,
    #[serde(skip_serializing)]
    pub all_messages_text: String,
    // Add from FTS:
    pub user_messages_text: String,
    pub assistant_messages_text: String,
    pub last_message: String,
    pub last_message_role: String,
    // future: is_favorite? Already in FTS types separately. Keep as separate field or compute.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsDiff {
    pub updated: Vec<SessionInfo>,
    pub removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTextSearchHit {
    pub session_id: String,
    pub session_path: String,
    pub session_name: Option<String>,
    pub entry_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTextSearchResponse {
    pub hits: Vec<FullTextSearchHit>,
    pub total_hits: usize,
    pub has_more: bool,
}
```

#### E. `src-tauri/src/config.rs`
- Ensure `Config` struct includes:
  - `enable_fts5: bool`
  - `realtime_cutoff_days: u64`
  - `session_paths: Vec<String>` (from main)
  - `transport: TransportConfig`? from main
  - `server: ServerConfig`? from main
  - etc.

#### F. `src-tauri/src/sqlite_cache.rs` (Complex)
**Approach:** Start with main's version as base, then incorporate FTS's schema additions and recovery.

**Schema merging:**
- CREATE TABLE sessions (main's columns + ALTER for user/assistant texts)
- ALTER TABLE sessions ADD COLUMN user_messages_text TEXT
- ALTER TABLE sessions ADD COLUMN assistant_messages_text TEXT
- CREATE TABLE message_entries (FTS)
- CREATE INDEX idx_message_entries_session ON message_entries(session_path)
- If `enable_fts5`:
  - CREATE VIRTUAL TABLE message_fts USING fts5(... with content='message_entries', content_rowid='rowid')
  - CREATE TRIGGER for updates on message_entries? Actually content= references another table; we need triggers to keep it in sync. FTS branch likely had trigger functions. We'll port those.
  - Also sessions_fts may exist from old versions; we may need to drop it or migrate. Keep if still used for session-level search.

- Keep main's other tables: tags, session_tag_assignments, favorites, settings_store, session_details_cache (if exists)
- Keep main's indexes: idx_modified, idx_cwd, idx_file_modified

- Keep FTS's `init_fts5` function and `ensure_message_fts_schema` for migrations
- Keep `full_rebuild_fts` for rebuilding message_fts
- Keep corruption recovery wrapper in `init_db_with_config`

**Sample merged logic:**
```rust
fn init_db_with_config(config: &Config) -> Result<Connection, String> {
    let db_path = get_db_path()?;
    // Open with recovery on corruption
    match open_and_init_db(&db_path, config) { ... }
}

fn open_and_init_db(db_path: &Path, config: &Config) -> Result<Connection, String> {
    let conn = Connection::open(db_path)?;
    // PRAGMA: WAL, synchronous, foreign_keys

    // Create main tables
    conn.execute("CREATE TABLE IF NOT EXISTS sessions (... long from main ...)", [])?;
    // ALTER for FTS fields
    conn.execute("ALTER TABLE sessions ADD COLUMN user_messages_text TEXT", []).ok();
    conn.execute("ALTER TABLE sessions ADD COLUMN assistant_messages_text TEXT", []).ok();

    // Create tag tables (main) ...

    // Create message_entries (FTS)
    conn.execute("CREATE TABLE IF NOT EXISTS message_entries (...)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_message_entries_session ON message_entries(session_path)", [])?;

    // FTS5
    if config.enable_fts5 {
        init_message_fts(&conn)?;
    }

    Ok(conn)
}
```

#### G. `src-tauri/src/scanner.rs`
**Take main's version** (with `SCAN_CACHE`, `write_buffer`, multi-path, incremental potential). Then:

- Modify `parse_session_info`:
  - After building `SessionInfo`, parse each line to extract messages
  - Insert into `message_entries` (session_path, id, role, content, timestamp)
  - Compute `user_messages_text` and `assistant_messages_text` by concatenating text parts
  - Update session's `user_messages_text`, `assistant_messages_text` fields **before** returning
- Ensure `parse_session_info` still returns `SessionInfo` with those fields populated

**How to get DB connection inside parser?** The scanner currently calls `parse_session_info` without DB context. We can either:
- Pass a connection reference to `parse_session_info`
- Or have `scan_sessions_with_config` call `upsert_session` which writes to DB; we could also call `upsert_message_entries` after parsing

Better: Modify `scan_sessions_with_config`:
```rust
if let Ok(info) = parse_session_info(&file_path) {
    // New: also parse and store messages
    let messages = extract_messages_from_file(&file_path)?;
    // Insert messages in transaction, replacing existing for this session
    sqlite_cache::upsert_message_entries(&conn, &info.path, &messages)?;
    // Also compute aggregates if needed (or compute on the fly from messages)
    let (user_text, assistant_text) = aggregate_message_texts(&messages);
    // Update info's fields
    let info = SessionInfo { user_messages_text: user_text, assistant_messages_text: assistant_text, ..info };
    sessions.push(info);
    write_buffer::buffer_session_write(&info, file_modified)?;
}
```

We'll need to add `sqlite_cache::upsert_message_entries(conn, session_path, messages)`.

**Corruption recovery:** Wrap the whole file processing loop with a retry mechanism similar to FTS's, but at a higher level: if any DB error indicates corruption, drop connection, delete DB, recreate, and restart scan from scratch (or skip the failing file). Since we're using write_buffer, errors might be queued; we need to handle flush errors.

**Incremental updates:** The scanner currently returns `Vec<SessionInfo>`. The main branch's `file_watcher` likely computes a diff by comparing with previous cache. That should remain unchanged; our scanner just updates the DB with message entries for each scanned session.

### 3.2 Frontend

#### H. `src/App.tsx`
**Adopt main's version** (with mobile, kanban, terminal, onboarding) as base. Then **add FTS modal**:

- Keep main's state: `mobileTab`, `showOnboarding`, `showTerminal`, `terminalConfig`, `viewMode`, `filterTagIds`, etc.
- Add back: `showFullTextSearch`, `initialEntryId`? Actually we'll use `scrollTargetId` now, so App doesn't need `initialEntryId`. Instead, Store/Context will hold the pending scroll target.
- Add `FullTextSearch` component import
- Add keyboard shortcut: `Cmd+Shift+F` → `setShowFullTextSearch(true)`
- In `handleSelectSession`, maybe clear a pending scroll target
- Render `<FullTextSearch isOpen={showFullTextSearch} onClose={...} onSelectResult={(session, entryId) => { setSelectedSession(session); setScrollTargetId(entryId); setShowFullTextSearch(false); }} />`
  - Need `setScrollTargetId`; this can be a function passed to SessionViewer via ref or context.
- Use main's `useIsMobile`, `useAppearance`, `useTags` hooks.

**How to pass scroll target to SessionViewer?**
Option: In App, keep state `pendingScrollTarget: string | null`. When FTS returns a result, set that state. When rendering `SessionViewer`, pass `initialScrollTarget={pendingScrollTarget}` and then clear it after SessionViewer handles it. Or use a callback ref: `sessionViewerRef.current?.scrollToEntry(entryId)`.

Let's check if main's SessionViewer already supports imperative scroll. It has `useEffect` watching `scrollTargetId`. So we can set `scrollTargetId` as a prop. In main's App, they don't pass it; it's internal. So we need to modify SessionViewer to accept `onScrollToEntry` or external control.

Simpler: Keep FTS's `initialEntryId` prop, and inside SessionViewer, when both `initialEntryId` and `scrollTargetId` conflict, use `initialEntryId` to set `scrollTargetId` internally. That preserves both patterns. But Rule 6 says drop `initialEntryId`. We can instead expose a method via ref.

Given complexity, it's acceptable to keep `initialEntryId` as a prop on SessionViewer for FTS use, while main uses internal state. The conflict in the file will be that main removed it. We'll re-add it specifically for FTS. That's okay - the integration branch is combining features.

But the instruction says "drop FTS's `initialEntryId` prop". Why? Because main already has equivalent. But main's equivalent is internal state, not a prop. So we could either:
- Keep `initialEntryId` prop (simpler)
- Or make `useScrollTarget` external via context

I'll keep `initialEntryId` for now to minimize changes. It's not harmful.

#### I. `src/components/SessionViewer.tsx`
- Start with main's version (has session content cache, scrollTargetId, onWebResume, etc.)
- Add `initialEntryId?: string` back to props interface
- Add `useEffect` that runs on mount: if `initialEntryId` set, call `setScrollTargetId(initialEntryId)`
- Keep main's `scrollTargetId` effect that scrolls and highlights

```typescript
interface SessionViewerProps {
  session: SessionInfo;
  initialEntryId?: string; // for search result navigation
  onExport: () => void;
  onRename: () => void;
  onBack?: () => void;
  onWebResume?: () => void;
  terminal?: TerminalType;
  piPath?: string;
  customCommand?: string;
}

// Inside component:
useEffect(() => {
  if (initialEntryId) {
    setScrollTargetId(initialEntryId);
  }
}, [initialEntryId]);
```

#### J. `src/components/FullTextSearch.tsx`
- Update to use `invoke` from `../transport` (not `@tauri-apps/api/core`)
- Update to use main's hooks if needed (maybe `useSessions` for some context? Not necessarily)
- Ensure pagination works with main's server (command still works)
- Keep all existing logic (sorting, filtering, role/glob)

#### K. `src/types.ts`
Add:
```typescript
export interface FullTextSearchHit {
  session_id: string;
  session_path: string;
  session_name?: string;
  entry_id: string;
  role: string;
  content: string;
  timestamp: string;
  score: number;
}

export interface FullTextSearchResponse {
  hits: FullTextSearchHit[];
  total_hits: number;
  has_more: boolean;
}
```
Also maybe keep `SessionsDiff` if defined here? Actually it's in Rust models; not needed in frontend unless used. The frontend `useSessions` might consume diff events from file watcher. Check if `useSessions` expects `SessionsDiff`. It likely does. So we need to define that type too:
```typescript
export interface SessionsDiff {
  updated: SessionInfo[];
  removed: string[];
}
```

#### L. i18n
- Create `src/i18n/locales/en-US/search.ts`:
```typescript
export default {
  fullText: {
    placeholder: "Full-text search...",
    noResults: "No results found",
    // ... keys from FTS's old search.ts
  },
  // other namespaces
};
```
- Copy to zh-CN, and to other locales (de, es, fr, ja) if they exist
- Remove old `src/i18n/locales/en-US/search.ts` if it conflicts

---

## Phase 3: Integration Testing

### 3.1 Build & Type Check
```bash
npm run build  # frontend
cargo check    # backend
# Fix any errors
```

### 3.2 Unit Tests (Rust)
```bash
cargo test --locked
```
Focus:
- `fts_message_integrity_test`
- `full_text_search_pagination_test`
- `migration_test` (update it to test combined schema)
- `search_test` (merge changes)
- `integration_test` (likely needs updates)

### 3.3 Frontend Dev Test
```bash
npm run tauri:dev
```
Check:
- FTS modal opens with Cmd+Shift+F
- Search returns results, pagination works
- Selecting result navigates to entry (scroll highlights)
- New UI features (kanban, terminal) work
- Mobile layout works
- Onboarding appears (if not completed)

### 3.4 Database Migration Test
- Take an old pre-FTS database (v0.1.x), run new app, ensure migration succeeds
- Take a pre-main DB with tags, ensure FTS fields added
- Take a pre-main DB with FTS (sessions_fts), ensure migration to message_entries works

### 3.5 Performance
- Full scan + message parsing: measure time with 1000 sessions
- Search query latency: should be fast with FTS5
- Incremental updates: modify a session, ensure message_entries updates without full rescans

---

## Phase 4: Polish & Documentation

### 4.1 Code Quality
```bash
cd src-tauri && cargo fmt && cargo clippy
# Fix any warnings
```

### 4.2 CHANGELOG
Add section:
```
## [Unreleased]
### Added
- Full-text search across message contents with FTS5
- Search result pagination and filtering by role/glob
- Jump-to-result from search modal
- Multi-path session scanning (from main)
- Kanban board view
- Embedded terminal panel
- Hierarchical tags
- Mobile-responsive UI
- Multi-language support (de, es, fr, ja)
... plus all other main features merged
```

### 4.3 README Updates
- Document FTS feature and its configuration
- Mention multi-path, kanban, tags, terminal, mobile
- Update screenshots

### 4.4 AGENTS.md
- Reflect new architecture: scanner → write_buffer → sqlite_cache, message_entries, FTS5
- Document command registration process
- Note dispatcher pattern (if any)

---

## Phase 5: Final Validation Checklist

- [ ] All Rust tests pass (including FTS-specific)
- [ ] Frontend builds without errors
- [ ] FTS modal opens, accepts queries, shows paginated results
- [ ] Clicking result scrolls to correct entry
- [ ] Can filter by role/glob and sort by score/date
- [ ] New sessions automatically indexed in message_entries
- [ ] Modified sessions update message_entries correctly
- [ ] Deleted sessions cascade delete messages
- [ ] Database corruption recovery works (simulate by corrupting DB)
- [ ] Multi-path scanning includes all configured directories
- [ ] Tags and tagging UI work
- [ ] Kanban drag-and-drop works
- [ ] Terminal panel opens and accepts input
- [ ] Mobile layout responsive
- [ ] CommandPalette still works (not broken by FTS)
- [ ] `cargo clippy` passes (fix any warnings)
- [ ] No console errors in dev console

---

## Rollback Plan

If integration becomes too problematic:
1. Keep `fts-integration-20250214` as experiment
2. Re-clone fresh from `origin/main`
3. Implement FTS as a clean-room feature using main's patterns from start (Option C from audit)
4. Use FTS branch as specification reference only

---

## Estimated Effort

| Phase | Estimated Time |
|-------|-----------------|
| Phase 0 (setup) | 0.5 day |
| Phase 1 (merge conflicts) | 2-3 days (most critical) |
| Phase 2 (scanner + DB) | 1-2 days |
| Phase 3 (frontend) | 1 day |
| Phase 4 (testing) | 1 day |
| Phase 5 (polish) | 0.5 day |
| **Total** | **5-8 days** |

---

## Success Criteria

- All mainline features preserved plus FTS working
- No regression in existing functionality
- Codebase passes all linting and tests
- Documentation up to date
- Ready for PR review

---

**Next Step:** Execute Phase 0, then commit the merge after conflict resolution and show a diff for review.
