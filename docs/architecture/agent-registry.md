<!-- SPDX-FileCopyrightText: 2026 Vũ Văn Tâm -->
<!-- SPDX-License-Identifier: Apache-2.0 -->

# Agent Registry Contract

The Agent Registry is a deterministic catalog of agent manifests. It does not load code, execute models, dispatch providers, or grant runtime authority.

## Manifest

Each manifest declares:

- stable lowercase identifier;
- semantic `major.minor.patch` version;
- display name;
- capabilities;
- permissions;
- required services.

## Invariants

- Agent IDs are unique.
- Registry iteration is ordered by ID.
- Capability matching requires every requested capability.
- Mutation proposal permission requires workspace read permission.
- Registration validates the manifest before state changes.
- Replacement is explicit; registration never silently overwrites an agent.

## Safety boundary

Capabilities describe what an agent can be selected for. Permissions describe what it may request. Neither grants direct access to the filesystem, shell, provider, or mutation APIs.

All execution still flows through Runtime, Orchestrator, Guard, approval, Sandbox, transaction, recovery, and Self Verification.

## Deferred

- manifest file parsing;
- signed plugin manifests;
- compatibility constraints;
- runtime service resolution;
- persistent registry state;
- dynamic agent loading.
