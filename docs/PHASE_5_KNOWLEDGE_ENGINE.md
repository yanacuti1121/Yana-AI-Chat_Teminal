# Phase 5 — Knowledge Engine

## Purpose

The Knowledge Engine organizes durable workspace evidence without using an LLM for memory construction or retrieval.

The model remains the reasoning layer. Knowledge retrieval is deterministic and bounded.

## Components

- **Timeline Tree** — session, task, and action order.
- **Entity Graph** — relationships among files, symbols, tests, decisions, commits, and receipts.
- **Evidence Index** — exact file and line references with provenance.
- **Fact Store** — replaceable, verifiable workspace facts.
- **Receipt Store** — append-only action outcomes linking files, decisions, evidence, and tests.
- **Retrieval Engine** — graph traversal plus evidence and receipt ranking.
- **Context Builder** — emits a bounded evidence block for the model.

## Zero-token rule

Knowledge creation and retrieval must not call a model.

```text
Workspace events
  -> Timeline / Entity / Evidence / Fact / Receipt
  -> Deterministic ranking
  -> Bounded context
  -> Model reasoning
```

## Safety boundaries

- Knowledge retrieval never mutates the workspace.
- Entity traversal has a hard depth limit.
- Context output has a hard character budget.
- Facts retain their source and observation time.
- Receipts are append-only.
- Tool calls remain data until approved through Operator, Forge, and Guard.

## Deferred

- Persistent knowledge snapshot format.
- Decision Graph adapter.
- Git commit and PR ingestion.
- Conflict filtering across evidence sources.
- Incremental knowledge updates from filesystem watchers.
- Query parser for natural-language entity resolution.
