# Visible UI Patch

This patch replaces the mock dashboard with a visibly different AI workspace UI.

## Visible changes

- conversation cards with role markers
- live workflow event panel
- switchable Activity / Compose / Zero-Memory panel (`Ctrl+M`)
- Compose lifecycle: Plan → Execute → Review → Test → Reflect
- Zero-token memory panel with original facts
- approval status in header and composer
- active sub-agent indicator
- rounded composer and richer status bar

## Commands

- `/help`
- `/compose`
- `/memory [query]`
- `/remember <fact>`
- `/agents`
- `/agent <main|planner|builder|reviewer|tester>`
- `/approve`
- `/reject`
- `/panel`

## Safety boundary

This remains a UI/workflow mock. It performs no host mutation and does not duplicate Yana Core authority.
