# Phase 2.5 — Operator Polish

Phase 2.5 turns the controlled action foundation into an explainable operator workflow.

## Pipeline

```text
Prompt
  -> Intent
  -> Scope suggestion
  -> Action preview
  -> Guard explanation
  -> Human approval
  -> Execution adapter (later)
  -> Evidence
  -> Verification
  -> Reflection
  -> Replay
```

## Added in this phase

- Intent classification with risk and suggested scope
- Dry-run action previews with impact levels
- Guard reports with reason, risk, and safer alternatives
- Journal replay using relative timeline offsets
- Structured post-task reflection
- Operator integration for intent, preview, guard, evidence, and reflection

## Safety boundary

This phase still does not grant a model direct filesystem or shell access. Forge describes and tracks actions; an execution adapter will be introduced only after sandbox, workspace-root validation, and explicit approval are connected.

## Product principle

Yana should not merely report that an action was blocked or completed. It should explain:

- what it understood;
- what it intends to do;
- why the action is necessary;
- what may be affected;
- what evidence verified the result;
- what should improve next time.
