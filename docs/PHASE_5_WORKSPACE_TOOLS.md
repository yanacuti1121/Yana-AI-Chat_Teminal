# Phase 5 — Real Workspace Tools

Phase 5 introduces the first host-facing adapters while preserving Yana's operator boundary.

## Included

- Workspace-confined UTF-8 file reads
- Bounded text search with line references
- Canonical path checks to prevent workspace escape
- Symlink-aware resolution for existing files
- Dry-run write plans
- Atomic temp-file replacement for approved writes
- Conservative command policy for read-only Git and test commands

## Safety invariants

1. Models never receive raw host filesystem access.
2. All paths are workspace-relative.
3. Existing paths are canonicalized before access.
4. Write parents are canonicalized before mutation.
5. Large files are rejected before reading.
6. Writes are prepared as previews before application.
7. Command policy does not execute processes; it only classifies requests.
8. Forge, Guard, Operator approval, and Journal receipts remain mandatory.

## Deferred

- Process execution adapter
- Full patch/hunk application
- Git mutation adapter
- Delete and rename adapters
- Rollback snapshots
- Binary file handling
