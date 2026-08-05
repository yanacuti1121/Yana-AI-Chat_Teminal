# Phase 4 — Workspace Intelligence

Phase 4 turns Yana from a capable operator shell into a workspace-aware development system.

## Modules

- `decision`: durable architecture decisions, scope, rationale, and supersession.
- `awareness`: operational signals for tests, index freshness, context pressure, model limitations, and pending decisions.
- `goal`: project goals that survive chat sessions and expose progress or blockers.
- `intelligence`: candidate ranking that combines symbols, dependencies, prior work, tests, and decision conflicts.

## Core behavior

Yana should prepare a small, explainable scope before asking a model to reason over the repository.

```text
Task
  -> Workspace facts
  -> Decision constraints
  -> Candidate ranking
  -> Recommended scope
  -> Operator approval
  -> Gateway route
```

## Safety boundary

Workspace Intelligence recommends; it does not mutate. All actions still pass through Operator, Forge, Guard, and explicit approval policy.

## Product principle

> Do not make the model wander through the repository. Prepare the workspace before the model enters it.
