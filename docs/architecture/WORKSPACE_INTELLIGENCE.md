# Workspace Intelligence Contract

Workspace Intelligence converts deterministic repository signals into a bounded operational map.

## Inputs

- dependency edges from Workspace Index or Atlas
- change and verification-failure counts
- compile and test durations
- explicit confidence values
- architectural zone classifications

No model call is allowed while constructing the map.

## Outputs

- stable workspace topology
- hotspot ranking with explicit reasons
- bounded reverse-dependency impact reports
- confidence metadata per module
- mutation eligibility derived from architectural zones

## Architectural zones

`Generated` and `ThirdParty` are never direct mutation targets. `Core`, `Stable`, and `Experimental` remain advisory classifications; Guard and approval still decide whether an action may proceed.

## Safety boundary

Workspace Intelligence does not read or mutate files, execute commands, approve actions, or override Guard, HALT, Sandbox, transaction, recovery, and verification policies.

## Determinism

Maps use ordered collections, hotspot ties are resolved by path, impact traversal is bounded by depth and node count, and the same signals must produce the same report.
