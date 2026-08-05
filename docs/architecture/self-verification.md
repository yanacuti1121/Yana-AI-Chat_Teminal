<!-- SPDX-FileCopyrightText: 2026 Vũ Văn Tâm -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Self Verification

Self Verification evaluates a proposed workspace change after execution evidence is available and before the result is accepted as complete.

## Pipeline

```text
Patch result
  -> Compile evidence
  -> Test evidence
  -> Static analysis evidence
  -> Project DNA compatibility
  -> Knowledge evidence linkage
  -> Scope discipline
  -> Recovery readiness
  -> Verification report
  -> Receipt
```

The verifier does not execute commands, read arbitrary files, mutate the workspace, or call a model. It evaluates structured evidence produced by the existing Operator, Workspace, Sandbox, Knowledge, Project DNA, and Recovery layers.

## Blocking checks

The default policy requires:

- compilation evidence;
- static-analysis evidence;
- tests for code changes;
- knowledge evidence identifiers;
- a rollback snapshot;
- changed-file scope below the configured limit.

A failed blocking check prevents the report from passing. A Project DNA conflict is a warning because inferred conventions are advisory, not an authority that can override user intent.

## Confidence

Confidence is deterministic and weight-based. Passed checks receive full weight, warnings receive half weight, and failed checks receive zero. Skipped checks are excluded from the denominator.

Confidence is not permission. A high confidence score cannot bypass Guard, approval, Sandbox, HALT, transaction, or recovery policy.

## Evidence boundary

The verifier stores compact evidence references and operational results. It does not store prompts, secrets, model chain-of-thought, or hidden summaries.

## Deferred integration

This sprint defines the verification engine and tests. Later integration will connect command receipts, concrete test adapters, Knowledge receipts, and terminal overlays without changing the authority boundary.
