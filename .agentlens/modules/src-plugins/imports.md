# Imports

[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)

## Dependency Graph

```mermaid
graph TD
    src_plugins[src-plugins] --> base[base]
    src_plugins[src-plugins] --> message[message]
    src_plugins[src-plugins] --> project[project]
    src_plugins[src-plugins] --> _[.]
    src_plugins[src-plugins] --> session[session]
    src_plugins[src-plugins] --> api[api]
    src_plugins[src-plugins] --> lucide_react[lucide-react]
```

## External Dependencies

Dependencies from other modules:

- `../base/BaseSearchPlugin`
- `./message/MessageSearchPlugin`
- `./project/ProjectSearchPlugin`
- `./registry`
- `./session/SessionSearchPlugin`
- `@tauri-apps/api/core`
- `lucide-react`

