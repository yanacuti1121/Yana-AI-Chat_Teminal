# Yana Terminal AI Workspace

A terminal-native AI operating workspace for developers.

This build focuses on the complete product surface discussed for Yana Terminal:

- richer transcript and composer UI
- Compose lifecycle: Plan → Execute → Review → Test → Reflect
- deterministic zero-token memory with original evidence
- working, session, project and decision memory classes
- main and sub-agent roles
- live workflow events
- explicit approval queue
- provider bridge boundary
- adaptive Activity / Plan / Memory side panel

The current provider/action flow remains a safe mock. It does not grant models direct shell or filesystem authority. Yana Core can be connected later through a bridge rather than duplicated here.

## Run

```bash
cargo run
```

## Keyboard

| Key | Action |
|---|---|
| `Ctrl+S` | Scope overlay |
| `Ctrl+P` | Plan overlay |
| `Ctrl+M` | Cycle Activity / Plan / Memory |
| `Tab` | Toggle focus/sidebar |
| `Esc` | Close overlay or quit |

See `CAPABILITIES.md` for commands and architecture boundaries.
