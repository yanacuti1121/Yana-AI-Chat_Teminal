# Phase 6 — Real Workspace Tools

Phase 6 completes the host-facing workspace layer while preserving Yana's operator boundary.

## Scope

- hunk-based text patches with exact context matching
- atomic rollback snapshots before mutation
- safe rename and delete plans
- workspace locks to prevent concurrent writes
- bounded directory listing and repository search
- deterministic diff previews

## Safety invariants

1. Every path is workspace-relative and canonicalized.
2. Symlink escapes are rejected.
3. Mutations require a prepared plan and explicit approval upstream.
4. Every mutation creates a rollback snapshot before changing the destination.
5. Concurrent mutation of the same path is rejected.
6. Commands remain policy-only; this phase does not add shell execution.
