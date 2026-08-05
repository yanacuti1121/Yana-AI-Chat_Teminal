# Phase 10.5 — Production Hardening

Phase 10.5 does not expand model authority or add a new user-facing capability. It turns the release foundation into a measurable production-readiness gate.

## Objectives

- Detect architecture boundary violations before merge.
- Evaluate task outcomes with deterministic, evidence-backed dimensions.
- Compare candidate releases against stored performance and quality baselines.
- Require recovery, rollback, provider, knowledge, security, licensing, and cross-platform checks.
- Keep publishing explicit and human-approved.

## Evaluation dimensions

Each task may be scored for correctness, safety, scope discipline, evidence quality, efficiency, and recovery. A task cannot pass when its verification tests fail, regardless of its aggregate score.

The evaluation layer stores measurements and evidence identifiers. It does not ask an LLM to judge itself.

## Regression policy

Release candidates are compared with a baseline for startup latency, Atlas indexing, retrieval latency, patch latency, peak memory, context size, provider tokens, and task score.

Metrics declare whether lower or higher values are better. A release is blocked only when regression exceeds the configured allowance. Missing data is reported rather than silently treated as success.

## Architecture boundaries

The default policy enforces these invariants:

- Provider adapters cannot access Workspace directly.
- UI cannot bypass Operator for mutation.
- Knowledge retrieval is read-only and does not invoke Provider adapters.
- Workspace mutation remains behind Operator, Core policy, and Recovery.
- Yana Core has no dependency on presentation layers.

## Required production checks

A production-ready report requires successful formatting, Clippy, tests, locked release build, SPDX headers, secret scan, recovery exercise, rollback exercise, provider contract checks, deterministic Knowledge checks, cross-platform builds, and documentation.

A reliability or benchmark score is informational until the corresponding required checks have passed. No score can override Guard, Sandbox, HALT, approval, or release policy.

## CI policy

The hardening workflow runs on macOS, Linux, and Windows. A separate policy job checks SPDX headers, likely committed secrets, and accidental automatic publishing commands.

The workflow builds artifacts but does not publish, sign, notarize, install, or execute an updater.

## Deferred production work

- Signed and notarized release artifacts
- Maintained Homebrew, MSI, deb, and rpm packages
- Cross-process mutation locks
- Long-running soak tests
- Fuzzing for parsers and path handling
- Real benchmark baselines gathered on controlled hardware
- External security review

These items are intentionally visible instead of being represented by empty stubs.
