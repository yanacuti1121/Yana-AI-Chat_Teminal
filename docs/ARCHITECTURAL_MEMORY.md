# Architectural Memory

Architectural Memory records why the workspace looks the way it does. It is not a chat summary and it does not ask a model to manufacture memory.

## Stored evidence

Each decision contains:

- a stable decision ID;
- status and scope;
- rationale;
- evidence IDs;
- attempted approaches and their outcomes;
- explicit supersession links;
- a deterministic timestamp supplied by the caller.

## Retrieval

Queries are deterministic and scope-aware. A scope ending in `/**` applies to that path and its descendants. Results are ordered by stable decision and approach identifiers.

## Conflict detection

Rejected or rolled-back approaches may produce advisory conflicts when a proposal repeats their explicit names inside the same scope. This is intentionally conservative: Architectural Memory does not infer hidden meaning and does not block execution by itself.

## Safety boundary

Architectural Memory:

- does not call a model;
- does not read or mutate the workspace;
- does not execute commands;
- cannot bypass Guard, approval, Sandbox, HALT, transactions, recovery, or verification;
- cannot silently replace an active decision;
- requires explicit supersession.

Persistence, receipt ingestion, terminal overlays, and integration with Adaptive Context and Self Verification remain separate follow-up work.
