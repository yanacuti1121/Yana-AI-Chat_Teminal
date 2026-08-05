# Phase 9 — Reliability & Recovery

Phase 9 makes failure explicit and recoverable without granting the model additional authority.

## Invariants

1. A workspace mutation belongs to one transaction.
2. Every applied step references a rollback snapshot.
3. Verification must finish before commit.
4. Rollback order is the reverse of applied order.
5. Session state is atomically persisted under `.yana/state/recovery.json`.
6. Telemetry remains local, bounded, and contains no prompt or secret values.
7. Reliability scoring is deterministic and never asks an LLM to judge health.
8. `HALT.lock` and Yana Core policy remain authoritative.

## Recovery state

The recovery snapshot records the current session, task, operator state, pending receipt, touched paths, and last update time. On restart, the terminal may offer resume or rollback. It must never silently continue a mutation.

## Transactions

The supported lifecycle is:

```text
Prepared → Applying → Verifying → Committed
                   ↘ RolledBack
```

Duplicate paths, partial apply, and partial verification are rejected.

## Reliability score

Component signals are normalized to 0–100 and combined deterministically:

- 90–100: Healthy
- 60–89: Degraded
- 0–59: Critical

This score is informational. It cannot bypass Guard, approval, sandbox, or HALT.

## Deferred

- startup resume overlay
- durable undo/redo command surface
- process watchdog and module restart
- OS-level file locks across processes
- checkpoint garbage collection
- low-memory policy integration with Context and Gateway
