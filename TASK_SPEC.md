# Full-Text Search (FTS) Subsystem – Implementation Tasks

## META Section

### 1. Overview & Mission
The Full‑Text Search subsystem enables users to quickly find relevant messages across all their AI coding sessions. It powers the “Cmd+Shift+F” modal, allowing filtering by role (user/assistant) and session path globs, with paginated results and highlighted snippets. The system must be **fast**, **reliable**, and **incrementally updated** as session files change.

### 2. Current State
- Uses SQLite FTS5 with a `message_entries` table and auto‑sync triggers.
- File watcher + scanner detect changes and update the database.
- Write buffer coalesces database writes for efficiency.
- Search command (`full_text_search`) builds queries with per‑session limits and glob filtering.

While functional, the current implementation has several correctness, performance, and robustness issues identified during the audit:
- Search results are ordered by insertion order, not relevance.
- FTS index may be left empty after schema recreation.
- Session files are parsed twice on updates, wasting I/O.
- No query timeouts; blocking DB calls run on async threads.
- Write buffer can grow unbounded.
- Per‑session limit picks the oldest messages, not the most recent.
- Role filter is case‑sensitive.
- Glob patterns are passed unsafely to SQL `GLOB`.
- Database corruption deletes everything without backup.
- Schema migrations use `.ok()` and may hide errors.
- Search semantics (phrase vs. OR) are unclear and differ from legacy search.

### 3. Strategic Goals
- **Correctness**: Search must return relevant results in a predictable order, respecting filters and recency.
- **Performance**: Searches should complete in <200ms for typical corpora (up to 10,000 sessions). Index updates should not block UI.
- **Reliability**: The system must survive file corruption, partial writes, and concurrent updates without data loss.
- **Maintainability**: Code should be clear, tested, and configurable for future evolution.

### 4. Key Design Decisions & Rationale
- **SQLite FTS5** – Chosen for its zero‑configuration, cross‑platform, and battle‑tested nature. Sufficient for desktop‑scale session archives.
- **Message‑level indexing** – Enables fine‑grained searches and accurate snippets.
- **Incremental updates** – File watcher + `rescan_changed_files` ensures low latency without full rescans.
- **Write buffer** – Reduces database write contention during batch operations.
- **Auto‑sync FTS triggers** – Guarantee index consistency without manual maintenance.

### 5. Non‑Goals
- Real‑time search as you type (search is triggered on submit).
- Full‑text search over skills/prompts (out of scope).
- Multi‑user or server‑side search (headless mode uses same DB).
- Advanced query syntax (AND/OR/parentheses) – may be added later, but not now.

### 6. Assumptions & Constraints
- Typical user corpus: ≤10,000 sessions, ≤500 messages each (≈5M messages).
- Session files are JSONL, each line a JSON object.
- File system events are reliable on major platforms (macOS, Windows, Linux).
- SQLite write lock contention is manageable with buffering.

### 7. Guiding Principles for Implementation
- **Relevance first** – Use FTS’s BM25 ranking by default.
- **Configurability** – Where sensible, expose knobs (per‑session limit, timeout, tokenizer) via settings.
- **Observability** – Add metrics for latency, cache hits, error rates.
- **Test coverage** – Every behavior must be covered by integration tests, including edge cases.

### 8. Cross‑cutting Concerns
- **Configuration** – New settings should be added to `config.toml` and exposed via frontend.
- **Metrics** – Use the `metrics` crate (or `tracing` with histogram) to capture search latency, query frequency, and index size.
- **Error handling** – All errors must be logged and, where safe, returned to the frontend for user visibility.

---

## TASK Section

### TASK-001: Fix Relevance Ranking in Search Results
**Rationale**: Currently, results are ordered by the `rowid` of the `message_entries` table (insertion order), not by the FTS5 relevance score. This makes searches far less useful.

**Description**:
- Modify the SQL in `commands/search.rs` to use the `rank` column provided by the FTS5 virtual table.
- Ensure the `rank` is propagated through the CTE and used in the final `ORDER BY`.
- Keep the per‑session limit, but the ranking should apply globally.

**Implementation Details**:
- In the `ranked` CTE, join `message_fts` and select `message_fts.rank` as `rank`.
- Remove the `m.rowid as rank` alias.
- After applying the per‑session limit, order by `rank` (which is FTS’s BM25 score, lower is better).
- Confirm that FTS5’s `rank` is lower for better matches; typical usage orders by `rank` ascending.

**Testing**:
- Add an integration test that inserts messages with different relevance (e.g., one message contains the query many times, another once) and verifies that the more relevant message appears first.
- Use the test helper `search_message_entries_with_params` from `full_text_search_command_test.rs` to inspect order.

**Dependencies**: None.

---

### TASK-002: Rebuild FTS Index After Virtual Table Recreation
**Rationale**: When `ensure_message_fts_schema` drops and recreates `message_fts` (e.g., because columns were missing), the new virtual table is empty. Existing messages become unsearchable until new inserts trigger the auto‑sync. We must rebuild the index immediately.

**Description**:
- After calling `create_message_fts5(conn)` in `ensure_message_fts_schema`, execute `INSERT INTO message_fts(message_fts) VALUES('rebuild');` to repopulate the index from `message_entries`.
- Ensure this only runs when the table was actually recreated, not on normal startup.

**Implementation Details**:
- In `ensure_message_fts_schema`, after the block that drops and recreates `message_fts`, check if the table was recreated (e.g., set a flag) and if so, run the rebuild command.
- The rebuild command may be slow for large databases; consider running it in a background thread or logging a warning.

**Testing**:
- Write a test that first creates a database with some messages, then corrupts the FTS by dropping shadow tables (as in `test_fts_vtable_corruption_triggers_database_recreation`), and then re‑runs `ensure_message_fts_schema`. Verify that the index is rebuilt and search returns results.
- Also test that a normal startup (without recreation) does not trigger rebuild.

**Dependencies**: None.

---

### TASK-003: Eliminate Double Parsing of Session Files During Updates
**Rationale**: `rescan_changed_files` calls `parse_session_info`, which returns parsed entries, but then `upsert_session` calls `insert_message_entries` again, re‑parsing the same file. This doubles I/O and CPU for every file change.

**Description**:
- Modify `upsert_session` to accept an optional slice of `SessionEntry`. If provided, use them to populate `message_entries` instead of re‑reading the file.
- In `rescan_changed_files`, after parsing, pass the entries to `upsert_session`.
- Keep the existing behavior (re‑parse) for callers that don’t have pre‑parsed entries (e.g., background scanner).

**Implementation Details**:
- Change `sqlite_cache::upsert_session` signature to `upsert_session(conn, session, file_modified, entries: Option<&[SessionEntry]>)`.
- Inside, if `entries` is `Some`, call `upsert_message_entries` directly; otherwise, call `insert_message_entries`.
- Update all call sites: `scanner::scan_sessions_with_config` (which currently doesn’t have entries) and `rescan_changed_files`.

**Testing**:
- Add a test that modifies a session file, triggers `rescan_changed_files`, and verifies that the search index is updated correctly. Use a mock or file system watcher to ensure no extra file reads occur (e.g., by counting `File::open` calls via a test wrapper – but that may be overkill). At minimum, test correctness.

**Dependencies**: None.

---

### TASK-004: Add Query Timeouts and Move Blocking DB Calls to `spawn_blocking`
**Rationale**: Long‑running or malformed queries can block the async Tokio runtime, degrading responsiveness. SQLite operations are blocking and should be offloaded to blocking threads.

**Description**:
- Wrap all database queries in `full_text_search` with `tokio::task::spawn_blocking`.
- Set a timeout on the overall search operation using `tokio::time::timeout`.
- Also set a `PRAGMA query_timeout` inside the blocking task for an additional safety net.

**Implementation Details**:
- Inside `full_text_search`, move the connection opening and query execution into a closure passed to `spawn_blocking`. The closure returns the result.
- Use `timeout(Duration::from_secs(5), spawn_blocking(...)).await` and handle timeout errors.
- Before executing the main queries, run `conn.execute("PRAGMA query_timeout = 5000", [])?` to limit query execution time at the SQLite level.
- Ensure that errors are properly propagated (e.g., `spawn_blocking` panics become `JoinError`; handle them).

**Testing**:
- Write a test that runs a deliberately slow query (e.g., using `LIKE '%a%'` on a large table) and verifies that the timeout triggers.
- Since timeouts are hard to test deterministically, a unit test with a very short timeout and a dummy query that sleeps using SQLite’s `sleep()` function (if available) may be used. Alternatively, rely on integration tests with realistic data and trust the mechanism.

**Dependencies**: None.

---

### TASK-005: Bound the Write Buffer to Prevent Memory Exhaustion
**Rationale**: The write buffer (`write_buffer`) can grow indefinitely if flushing repeatedly fails, leading to unbounded memory usage.

**Description**:
- Introduce a maximum capacity for the buffer (e.g., 1000 entries for sessions, 1000 for details).
- When the capacity is reached, apply backpressure: either block the writer or drop oldest entries.

**Implementation Details**:
- Add constants `MAX_SESSION_BUFFER` and `MAX_DETAILS_BUFFER` (configurable via `Config`).
- In `buffer_session_write` and `buffer_details_write`, check the current size and either:
  - If under the limit, insert.
  - If at limit, either drop the oldest entry (based on `cached_at`) or return an error (which would be logged). Prefer dropping oldest to avoid blocking the scanner.
- Update `check_and_take_flush_data` to still flush all entries regardless of size.
- Add metrics to monitor buffer size.

**Testing**:
- Create a test that fills the buffer beyond capacity and verifies that older entries are evicted (or that writes block/error as designed). Since the buffer is static, use a mock or adjust the constants for testing.

**Dependencies**: None.

---

### TASK-006: Fix Per‑Session Limit to Use Most Recent Messages
**Rationale**: The current per‑session limit picks the first 3 messages by `rowid` (insertion order), which may return very old messages and hide recent ones. Users expect to see the most recent matches.

**Description**:
- In the `ranked` CTE, change the `PARTITION BY` order from `m.rowid` to `m.timestamp DESC` (or `m.rowid DESC` if timestamp is monotonic, but timestamp is safer).
- Ensure the `rn_in_session` column now ranks messages from newest to oldest.

**Implementation Details**:
- Modify the SQL in `commands/search.rs`:
  ```sql
  ROW_NUMBER() OVER (PARTITION BY m.session_path ORDER BY m.timestamp DESC) as rn_in_session
  ```
- Keep the global ordering by `rank` (after fix in TASK-001).
- Verify that the final result set contains the most recent matches from each session.

**Testing**:
- Extend `full_text_search_pagination_test.rs` to include messages with different timestamps and assert that the per‑session limit picks the most recent ones.
- Also test that when a session has fewer than 3 matches, all are returned.

**Dependencies**: TASK-001 (to ensure correct ranking order is not interfered).

---

### TASK-007: Implement Safe Backup Before Destructive Database Recovery
**Rationale**: When SQLite corruption is detected, the current code deletes the database file without saving a backup, causing loss of all user data (tags, favorites, settings). We must preserve a copy for potential recovery.

**Description**:
- Before deleting a corrupted database in `init_db_with_config`, copy the file to a backup location with a timestamp.
- Log a warning and optionally notify the user (via event or toast) that corruption was detected and a backup was created.

**Implementation Details**:
- In the error recovery branch of `init_db_with_config`, after detecting corruption and before calling `fs::remove_file`, generate a backup path: `db_path.with_extension("db.corrupted.<timestamp>")`.
- Use `fs::copy` to copy the corrupted file.
- If copy succeeds, log an info message; if it fails, log an error but proceed with deletion.
- After successful copy, delete the original.
- Consider adding a configuration option to disable backup or set backup directory.

**Testing**:
- Write a test that creates a valid database, then corrupts it by writing garbage, runs `init_db_with_config`, and verifies that a backup file exists and the original is gone.
- Use `tempfile` to avoid touching real user data.

**Dependencies**: None.

---

### TASK-008: Normalize Role Filter Input
**Rationale**: The role filter expects exact strings `"user"` or `"assistant"`. If the frontend sends `"User"` (capitalized), the filter silently becomes `"all"` (due to fallback), misleading users.

**Description**:
- In `full_text_search`, convert the incoming `role_filter` string to lowercase before using it in the SQL condition.
- Also update the `RoleFilter` enum in `search.rs` (legacy search) to be case‑insensitive, though the legacy search is less critical.

**Implementation Details**:
- In `commands/search.rs`, after extracting `role_filter`, map it: `let role_filter = role_filter.to_lowercase();`
- Then match on `role_filter.as_str()` to set `role_opt`.
- Ensure that any unexpected value still results in `None` (all).

**Testing**:
- Add a test case in `full_text_search_command_test.rs` that sends `"User"` (uppercase) and verifies that only user messages are returned.

**Dependencies**: None.

---

### TASK-009: Decide and Implement Consistent Search Semantics (Phrase vs. OR)
**Rationale**: The current FTS implementation treats the entire query as a phrase (wrapped in double quotes). The legacy `search_sessions` used OR semantics (split by whitespace). This inconsistency may confuse users. We need to decide on a default and make it configurable.

**Description**:
- Evaluate user expectations: most search boxes treat space as AND (inclusive) or OR? Typically, space is AND, but FTS5 default is OR for multiple terms. The current phrase search requires exact word order, which is too restrictive.
- Propose: Default to **OR** (match any word) because it’s more likely to find relevant messages. Allow switching to **phrase** or **AND** via a UI toggle or configuration.
- Implement a `search_mode` parameter in `full_text_search` (default `"any"`) that controls how the query is built:
  - `"any"` (OR): `word1 OR word2 OR word3`
  - `"all"` (AND): `word1 AND word2 AND word3`
  - `"phrase"`: `"word1 word2 word3"` (exact order)
- Keep the existing escaping logic.

**Implementation Details**:
- Add a new optional parameter `match_mode` (string) to the command.
- In `full_text_search`, based on `match_mode`, construct the FTS MATCH string:
  - Split the query into words (by whitespace) and join with appropriate operators.
  - For `"any"`: `word1 OR word2 ...`
  - For `"all"`: `word1 word2 ...` (FTS5 treats consecutive words as AND by default? Actually, in FTS5, `word1 word2` means word1 AND word2. So we can simply join with spaces for AND.)
  - For `"phrase"`: wrap the whole escaped query in quotes.
- Ensure that the escaping logic is applied per word for OR/AND modes (escape special chars but not the operators).

**Testing**:
- Extend `full_text_search_command_test.rs` with cases for each mode, using messages that contain different word combinations.
- Also test that special characters are handled correctly in each mode.

**Configuration**:
- Add a setting `search.match_mode` with options `"any"`, `"all"`, `"phrase"`. Default to `"any"`.
- Frontend can expose a dropdown in search modal.

**Dependencies**: None.

---

### TASK-010: Escape Glob Patterns or Use Safer Alternative
**Rationale**: User‑supplied glob patterns are passed directly to SQL `GLOB`, which interprets `*`, `?`, `[` as wildcards. A pattern like `[*]` could cause unintended matches or even errors. Moreover, `GLOB` with a leading wildcard cannot use indexes.

**Description**:
- Replace `GLOB` with `LIKE` and escape the user‑supplied pattern properly (escape `%` and `_`).
- Alternatively, if we keep `GLOB`, at least escape special characters by doubling them (e.g., `*` → `[*]`) – but that’s complex.
- Safer: Use `LIKE` and escape. However, `LIKE` is case‑insensitive by default in SQLite, which may be acceptable for paths.
- Also consider adding an index on `session_path` (already exists) and note that `LIKE` with a prefix (no leading `%`) can use the index.

**Implementation Details**:
- In `commands/search.rs`, when building the `where_clause`, if a glob pattern is provided, convert it to a `LIKE` pattern:
  - Replace `*` with `%`
  - Replace `?` with `_`
  - Escape any existing `%` and `_` in the original pattern by prefixing with `\` and set `ESCAPE '\'`.
- Alternatively, keep `GLOB` but escape the pattern by doubling special characters (e.g., `*` → `[*]`) – this is more aligned with user expectations of glob syntax.
- Document that `GLOB` may be slow for non‑prefix patterns.

**Testing**:
- Add test cases for glob patterns containing special chars (e.g., `*`, `?`, `[abc]`) and verify that the search correctly matches or escapes them.
- Test that a pattern like `**/project/**` still works as expected.

**Dependencies**: None.

---

### TASK-011: Improve Schema Migrations with Versioning and Robust Error Handling
**Rationale**: The current code uses many `ALTER TABLE` statements with `.ok()`, ignoring potential errors. There is no version table, so migrations are re‑run every startup, which is inefficient and hides failures.

**Description**:
- Introduce a `db_version` table (key‑value) to store the current schema version.
- On startup, read the version and apply migrations in a controlled, transactional manner.
- Remove ad‑hoc `ALTER TABLE` calls and centralize migration logic.

**Implementation Details**:
- Create a new table `schema_version` with columns `version INTEGER PRIMARY KEY, applied_at TEXT`.
- In `init_db_with_config`, after ensuring all base tables exist, read the current version (default 0).
- Define a list of migration steps, each with a version number and SQL to execute (may contain multiple statements). Run them in order inside a transaction.
- After each successful migration, update the version.
- Remove all the scattered `ALTER TABLE` calls and the `.ok()` swallowers.
- Ensure that the FTS tables are also handled in migrations.

**Testing**:
- Write tests that create a database with an old schema (e.g., from a dump), run the migrations, and verify that all required columns and indexes exist.
- Test that running migrations multiple times is idempotent.

**Dependencies**: None, but this is a significant refactor.

---

### TASK-012: Add Indexes for Performance (If Missing)
**Rationale**: The current schema already has indexes on `modified`, `cwd`, `file_modified`, and `session_path` in `message_entries`. However, the search query uses `session_path` in the `PARTITION BY` and may benefit from a covering index.

**Description**:
- Review the query execution plan for the paginated search and ensure appropriate indexes exist.
- Consider adding a composite index on `(session_path, timestamp)` to speed up the per‑session ordering.

**Implementation Details**:
- Add a migration (see TASK-011) to create `CREATE INDEX idx_message_entries_session_time ON message_entries(session_path, timestamp);`
- Verify with `EXPLAIN QUERY PLAN` that the index is used.

**Testing**:
- No specific test needed, but performance benchmarks can validate improvement.

**Dependencies**: TASK-011 (to add index via migration).

---

### TASK-013: Add Metrics for Observability
**Rationale**: Currently, there is no insight into search performance, cache effectiveness, or error rates. Adding metrics will help diagnose issues and guide future optimizations.

**Description**:
- Integrate the `metrics` crate (or use `tracing` with histograms) to capture:
  - Number of search queries (counter).
  - Query latency (histogram).
  - Number of results (histogram).
  - Index size (gauge) – number of messages in `message_entries`.
  - Write buffer size (gauge).
  - Database corruption recovery events (counter).
- Expose metrics via an endpoint (e.g., `/metrics` in HTTP adapter) for Prometheus scraping.

**Implementation Details**:
- Add dependencies: `metrics`, `metrics-exporter-prometheus` (optional).
- In `full_text_search`, record latency and result count.
- In `write_buffer`, record buffer sizes.
- In `init_db_with_config`, increment corruption counter.
- In the HTTP adapter, add a route `/metrics` that returns Prometheus format if the exporter is enabled.

**Testing**:
- Unit tests can verify that metrics are recorded (e.g., by using a test recorder). Integration tests can check that the `/metrics` endpoint returns data.

**Configuration**:
- Add a config option `metrics.enabled` (default false) and `metrics.prometheus_port` (optional). In headless mode, start a separate metrics server.

**Dependencies**: None.

---

### TASK-014: Write High‑Power Integration Tests for All Behaviors
**Rationale**: To ensure the fixes work correctly and prevent regressions, we need comprehensive integration tests covering each new behavior and invariant.

**Description**:
- Extend existing test files (`full_text_search_integration_test.rs`, `fts_message_integrity_test.rs`, `full_text_search_command_test.rs`) to cover:
  - Relevance ordering (TASK-001).
  - FTS rebuild after recreation (TASK-002).
  - No double parsing (TASK-003) – though hard to assert directly, we can test that updates work correctly.
  - Query timeouts (TASK-004) – maybe simulate a slow query.
  - Write buffer bounds (TASK-005) – test eviction.
  - Per‑session limit with recent messages (TASK-006).
  - Backup on corruption (TASK-007).
  - Role filter case insensitivity (TASK-008).
  - Search modes (TASK-009).
  - Glob escaping (TASK-010).
  - Schema migrations (TASK-011).
  - Index usage (TASK-012) – not directly testable, but explain.
  - Metrics (TASK-013) – test endpoint.
- For each, write a dedicated test function with clear setup, action, and assertion.

**Implementation Details**:
- Use temporary directories and databases for isolation.
- Leverage `tempfile` and set `HOME` to temp dir to avoid polluting real user data.
- Where needed, use `std::panic::catch_unwind` to ensure cleanup even on test failure.
- For tests that require file system events, use `notify` with a short polling interval or just call `rescan_changed_files` directly.

**Dependencies**: All previous tasks, as they introduce new behavior to test.

---

### TASK-015: Document New Features and Configuration Options
**Rationale**: Users and developers need to understand the search capabilities and how to configure them.

**Description**:
- Update the README with a section on Full‑Text Search, describing:
  - How it works (message‑level indexing, incremental updates).
  - Supported filters (role, glob).
  - Search modes (any, all, phrase) and how to switch.
  - Performance tips.
- Add comments in `config.toml.example` for new settings (match mode, timeout, buffer size, metrics).
- Update the frontend help text if necessary.

**Implementation Details**:
- Markdown updates in `README.md` and `src-tauri/README.md` (if exists).
- Add inline documentation for public functions.

**Dependencies**: All tasks that introduce configurable features.

---

## Summary of Tasks

| ID       | Title                                      | Priority | Dependencies       |
|----------|--------------------------------------------|----------|--------------------|
| TASK-001 | Fix relevance ranking                      | P0       | None               |
| TASK-002 | Rebuild FTS after recreation                | P0       | None               |
| TASK-003 | Eliminate double parsing                    | P1       | None               |
| TASK-004 | Add query timeouts + spawn_blocking         | P1       | None               |
| TASK-005 | Bound write buffer                          | P2       | None               |
| TASK-006 | Fix per‑session limit ordering              | P1       | TASK-001           |
| TASK-007 | Backup on corruption                        | P0       | None               |
| TASK-008 | Normalize role filter                       | P2       | None               |
| TASK-009 | Implement configurable search modes         | P2       | TASK-001           |
| TASK-010 | Escape glob patterns                        | P2       | None               |
| TASK-011 | Improve schema migrations with versioning   | P1       | None               |
| TASK-012 | Add indexes                                 | P2       | TASK-011           |
| TASK-013 | Add metrics                                 | P2       | None               |
| TASK-014 | Write high‑power integration tests          | P0       | All previous       |
| TASK-015 | Documentation                               | P2       | TASK-009, TASK-013 |

---

**End of Document**