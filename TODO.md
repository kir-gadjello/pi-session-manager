# TODO - Pi Session Manager

## Full-Text Search (FTS) Subsystem Tasks

### P0 - Critical (Fix immediately)

- [ ] TASK-001: Fix relevance ranking in search results (use FTS5 BM25 rank)
- [ ] TASK-002: Rebuild FTS index after virtual table recreation
- [ ] TASK-007: Implement safe backup before destructive database recovery
- [ ] TASK-014: Write high-power integration tests for all behaviors

### P1 - High Priority

- [ ] TASK-003: Eliminate double parsing of session files during updates
- [ ] TASK-004: Add query timeouts and move blocking DB calls to spawn_blocking
- [ ] TASK-006: Fix per-session limit to use most recent messages (depends on TASK-001)
- [ ] TASK-011: Improve schema migrations with versioning and robust error handling
- [ ] TASK-012: Add indexes for performance (depends on TASK-011)

### P2 - Medium/Low Priority

- [ ] TASK-005: Bound the write buffer to prevent memory exhaustion
- [ ] TASK-008: Normalize role filter input (case-insensitive)
- [ ] TASK-009: Implement consistent search semantics (phrase vs OR) with configurable match mode
- [ ] TASK-010: Escape glob patterns or use safer alternative
- [ ] TASK-013: Add metrics for observability
- [ ] TASK-015: Document new features and configuration options

---

## Notes

- All tasks are defined in `TASK_SPEC.md`.
- Tasks should be implemented in priority order, respecting dependencies.
- Integration tests are mandatory for each feature (see TASK-014).
- The FTS system must remain functional throughout all changes.
