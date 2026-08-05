# Phase 6 — Persistent Memory and Real Atlas

Phase 6 makes project knowledge survive terminal restarts and turns Atlas into a bounded repository index.

## Persistent Memory

Memory is stored in a versioned JSON document under `.yana/state/memory.json`.

Supported memory classes:

- working
- project
- decision
- user
- tool
- pattern
- conversation

Entries may be project-scoped and may carry an expiration time. Expired working memory is pruned explicitly rather than silently reused.

Writes use a temporary file followed by rename so an interrupted serialization does not leave a partially written state document.

## Real Atlas

Atlas scans a bounded set of UTF-8 source files and records:

- files/modules
- symbols and source lines
- top-level dependencies
- test markers
- stable content hashes
- reverse dependency queries

The first parser deliberately focuses on Rust structure and lightweight text metadata. It does not claim semantic equivalence to rust-analyzer or an LSP.

## Limits

Indexing is bounded by:

- maximum file count
- maximum bytes per file
- supported text extensions
- ignored build/vendor directories
- no symlink traversal

These limits prevent a local model request from causing an uncontrolled full-disk scan.

## Project State Layout

```text
.yana/
└── state/
    ├── memory.json
    └── atlas.json
```

`ProjectStores` owns these paths and coordinates loading and flushing both documents.

## Safety Boundary

Persistence and indexing do not mutate source files. Source changes still require:

```text
Operator → Forge → Guard → Preview → Approval → Workspace adapter
```

Atlas provides evidence and scope candidates only. Memory provides durable context only. Neither may bypass action policy.

## Deferred

- tree-sitter and LSP adapters
- incremental file watcher updates
- SQLite backend and migrations
- encrypted user-memory storage
- Git-history graph
- cross-project global memory policy
