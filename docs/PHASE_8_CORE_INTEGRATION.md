# Phase 8 — Yana Core Integration

Phase 8 turns the terminal into a client of the existing Yana Core instead of creating parallel Guard, Doctor, Audit, Sandbox, Skills, Budget, or HALT implementations.

## Integration model

```text
Yana Core
├── Guard
├── Doctor
├── Audit
├── Sandbox
├── Skills
├── Token Budget
└── HALT

        ▲
        │ capability facade
        │
Yana Terminal
```

## Added

- `YanaCore::discover` locates an existing Yana installation through `bin/yana`.
- `CapabilityRegistry` detects which shared services are present.
- `HaltGuard` checks the shared HALT markers before any action path proceeds.
- `CoreDoctor` produces a read-only command plan and parses returned output.
- `CoreReceipt` normalizes terminal receipts for the existing audit/knowledge path.

## Safety boundary

The terminal does not execute arbitrary shell text. Core commands are represented as structured plans. Execution must still pass through the sandbox and operator approval path.

The terminal does not auto-clear `HALT.lock`.

Provider adapters cannot call Core services directly. Tool calls remain untrusted data until the Operator authorizes them.

## Follow-up integration

The next implementation layer should add concrete adapters for:

1. the existing Yana audit-chain schema;
2. the existing sandbox executor;
3. the existing token-budget status output;
4. the existing skill registry;
5. terminal overlays fed from `DoctorSnapshot` and `CapabilityRegistry`.
