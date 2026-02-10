# Outline

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

Symbol maps for 4 large files in this module.

## src-tauri/src/sqlite_cache.rs (564 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 11 | fn | get_db_path | pub |
| 18 | fn | init_db | pub |
| 23 | fn | init_db_with_config | pub |
| 87 | type | TEXT | (private) |
| 113 | fn | init_fts5 | (private) |
| 156 | fn | upsert_session | pub |
| 189 | fn | get_session | pub |
| 221 | fn | get_all_sessions | pub |
| 248 | fn | get_sessions_modified_after | pub |
| 275 | fn | get_sessions_modified_before | pub |
| 302 | fn | get_cached_file_modified | pub |
| 314 | fn | delete_session | pub |
| 320 | fn | get_session_count | pub |
| 327 | struct | SessionDetailsCache | pub |
| 342 | fn | get_session_details_cache | pub |
| 371 | fn | upsert_session_details_cache | pub |
| 419 | fn | vacuum | pub |
| 425 | fn | cleanup_missing_files | pub |
| 446 | fn | preload_recent_sessions | pub |
| 475 | fn | search_fts5 | pub |
| 491 | fn | optimize_database | pub |
| 498 | fn | parse_timestamp | (private) |
| 506 | struct | DbFavoriteItem | pub |
| 515 | fn | add_favorite | pub |
| 523 | fn | remove_favorite | pub |
| 529 | fn | get_all_favorites | pub |
| 549 | fn | is_favorite | pub |
| 555 | fn | toggle_favorite | pub |

## src-tauri/tests/search_test.rs (522 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 7 | fn | create_test_session_file | (private) |
| 13 | fn | cleanup_test_dir | (private) |
| 20 | fn | test_empty_query_returns_empty_results | (private) |
| 60 | fn | test_single_word_search | (private) |
| 102 | fn | test_multiple_word_search | (private) |
| 143 | fn | test_name_search_mode | (private) |
| 184 | fn | test_role_filter | (private) |
| 231 | fn | test_multiple_sessions | (private) |
| 314 | fn | test_snippet_generation | (private) |
| 350 | fn | test_score_calculation | (private) |
| 406 | fn | test_thinking_content | (private) |
| 443 | fn | test_empty_sessions_list | (private) |
| 451 | fn | test_special_characters | (private) |
| 488 | fn | test_unicode_search | (private) |

## src/components/SessionTree.tsx (730 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 7 | fn | highlightText | (private) |
| 20 | fn | extractSnippet | (private) |
| 45 | interface | SessionTreeRef | pub |
| 49 | interface | SessionTreeProps | (private) |
| 56 | interface | TreeNodeData | (private) |
| 62 | interface | FlatNode | (private) |
| 121 | fn | sortChildren | (private) |
| 159 | fn | markActive | (private) |
| 255 | fn | isSettingsEntry | (private) |
| 595 | fn | handleNodeClick | (private) |

## src/components/SessionViewer.tsx (708 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 21 | interface | SessionViewerProps | (private) |
| 37 | fn | SessionViewerContent | (private) |
| 94 | fn | doLoad | (private) |
| 171 | fn | handleScroll | (private) |
| 181 | fn | checkFileChanges | (private) |
| 207 | fn | handleKeyDown | (private) |
| 321 | fn | tryHighlight | (private) |
| 345 | fn | handleScroll | (private) |
| 387 | fn | handleMouseMove | (private) |
| 397 | fn | handleMouseUp | (private) |
| 412 | fn | loadIncremental | (private) |

