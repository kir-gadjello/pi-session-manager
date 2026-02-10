# Imports

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Dependency Graph

```mermaid
graph TD
    src_hooks[src-hooks] --> contexts[contexts]
    src_hooks[src-hooks] --> __[..]
    src_hooks[src-hooks] --> plugins[plugins]
    src_hooks[src-hooks] --> types[types]
    src_hooks[src-hooks] --> _[.]
    src_hooks[src-hooks] --> _[.]
    src_hooks[src-hooks] --> _[.]
    src_hooks[src-hooks] --> api[api]
    src_hooks[src-hooks] --> api[api]
    src_hooks[src-hooks] --> _tauri_apps[@tauri-apps]
    src_hooks[src-hooks] --> date_fns[date-fns]
    src_hooks[src-hooks] --> react[react]
    src_hooks[src-hooks] --> react_i18next[react-i18next]
```

## External Dependencies

Dependencies from other modules:

- `../contexts/SettingsContext`
- `../i18n`
- `../plugins/registry`
- `../types/settings`
- `./useDemoMode`
- `./useSearchCache`
- `./useSettings`
- `@tauri-apps/api/core`
- `@tauri-apps/api/event`
- `@tauri-apps/plugin-dialog`
- `date-fns`
- `react`
- `react-i18next`

