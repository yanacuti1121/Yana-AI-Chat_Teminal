<!-- SPDX-FileCopyrightText: 2026 Vũ Văn Tâm -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Agent Runtime Contract

The Agent Runtime executes already-approved work. It does not decide what the user intended and it does not call providers directly.

## Responsibilities

- own session and task lifecycle state;
- schedule queued tasks deterministically by priority and task ID;
- pause, resume, stop, cancel, complete, and fail work;
- emit bounded structured runtime events;
- expose derived local metrics.

## Boundaries

The runtime cannot bypass Guard, approval, Sandbox, `HALT.lock`, transactions, recovery, or self-verification. It contains no model calls, shell execution, filesystem mutation, network access, or hidden autonomous continuation.

## State model

Sessions move between `Running`, `Paused`, and `Stopped`. Tasks move through `Queued`, `Running`, and a terminal state. A stopped session cancels pending or running tasks and cannot accept new work.

## Determinism

Priority ordering is stable: higher priority runs first, then lower task ID. Paused sessions retain queued work but cannot dispatch it. Runtime metrics are derived from current task state rather than increment-only counters.
