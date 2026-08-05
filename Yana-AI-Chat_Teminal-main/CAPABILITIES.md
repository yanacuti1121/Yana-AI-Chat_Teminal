# Yana Terminal Capabilities

This build groups the terminal around product capabilities instead of adding isolated subsystems.

## UI workspace
Chat transcript, richer composer, adaptive side panel, live workflow events, scope and plan overlays.

## Compose mode
Deterministic lifecycle: Plan → Execute → Review → Test → Reflect. The current build simulates the lifecycle and never performs host mutation directly.

## Zero-token memory
Original facts are indexed by entity text and timestamp. Retrieval uses deterministic matching and ordering; no LLM summary is required for memory operations.

Memory classes: working, session, project and decision.

## Agent surface
Main, planner, builder, reviewer and tester roles. They are UI/runtime roles only; no autonomous host authority is granted.

## Approval and streaming
Actions can enter an explicit approval queue. Streaming state is represented independently so a real provider bridge can replace the mock workflow later.

## Commands

- `/compose`
- `/memory [query]`
- `/remember <fact>`
- `/agents`
- `/agent <main|planner|builder|reviewer|tester>`
- `/approve`
- `/reject`
- `/panel`
- `/provider [name]`
- `/search <text>`
- `/attach <path>`
- `/theme`
- `/save`
