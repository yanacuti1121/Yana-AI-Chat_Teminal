# Project DNA

Project DNA extracts repository conventions from repeatable source evidence. It is not a model-generated style summary.

## Inputs

- workspace-indexed source files
- bounded line-level evidence
- deterministic rule detectors

## Outputs

Each convention contains:

- convention category
- normalized rule
- support count
- confidence percentage
- bounded source evidence

## Current convention categories

- naming
- error handling
- testing
- documentation
- module layout

## Boundaries

- Project DNA performs no model calls.
- It does not mutate the workspace.
- A convention is omitted until it reaches minimum support.
- Evidence is retained so every rule can be inspected.
- Detection results may advise verification and context ranking, but cannot bypass Guard, approval, Sandbox, HALT, or recovery policy.
- Weak evidence must never be converted into a mandatory repository rule.

## Intended flow

```text
Workspace Index
      |
      v
Project DNA inference
      |
      +--> Context ranking hints
      +--> Proposal conflict checks
      +--> Verification policy hints
```

Future language-specific detectors should remain separate from enforcement. Enforcement belongs to the Operator and verification pipeline, with human-reviewable evidence.
