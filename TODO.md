# TODO - Pi Session Manager

## Full-Text Search (FTS) Subsystem Tasks

### P0 - Critical (Fix immediately)

- [x] TASK-001: Fix relevance ranking in search results (use FTS5 BM25 rank)
- [x] TASK-002: Rebuild FTS index after virtual table recreation
- [x] TASK-007: Implement safe backup before destructive database recovery
- [x] TASK-014: Write high-power integration tests for all behaviors

### P1 - High Priority

- [x] TASK-003: Eliminate double parsing of session files during updates
- [x] TASK-004: Add query timeouts and move blocking DB calls to spawn_blocking
- [x] TASK-006: Fix per-session limit to use most recent messages (depends on TASK-001)
- [x] TASK-011: Improve schema migrations with versioning and robust error handling
- [x] TASK-012: Add indexes for performance (depends on TASK-011)

### P2 - Medium/Low Priority

- [x] TASK-005: Bound the write buffer to prevent memory exhaustion
- [x] TASK-008: Normalize role filter input (case-insensitive)
- [x] TASK-009: Implement consistent search semantics (phrase vs OR) with configurable match mode
- [x] TASK-010: Escape glob patterns or use safer alternative
- [x] TASK-013: Add metrics for observability
- [x] TASK-015: Document new features and configuration options

---

## Notes

- All tasks are defined in `TASK_SPEC.md`.
- Tasks should be implemented in priority order, respecting dependencies.
- Integration tests are mandatory for each feature (see TASK-014).
- The FTS system must remain functional throughout all changes.
