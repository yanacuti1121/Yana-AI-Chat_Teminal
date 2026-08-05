# Build notes

Implemented in this package:

- improved transcript/composer visual hierarchy
- Activity / Plan / Memory side-panel cycling with `Ctrl+M`
- Compose lifecycle state machine
- deterministic zero-token memory using original facts, entity text and timeline ordering
- working/session/project/decision memory classes
- main/planner/builder/reviewer/tester agent roles
- explicit approval queue
- workflow event stream
- streaming state placeholder for a real provider bridge

Validation note: the artifact environment did not include a Rust toolchain, so run the following locally before committing:

```bash
cargo fmt --all
cargo check
cargo test --all-targets
cargo run
```
