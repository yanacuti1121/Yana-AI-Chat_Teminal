# Phase 10 — Release & Packaging

Phase 10 turns the terminal prototype into a reproducible release candidate.

## Release contract

A release must pass:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets`
- `cargo build --release --locked`
- macOS, Linux, and Windows verification

## Artifact naming

Artifacts use the form:

```text
yana-terminal-v<version>-<rust-target>.<archive>
```

Examples:

```text
yana-terminal-v1.0.0-aarch64-apple-darwin.tar.gz
yana-terminal-v1.0.0-x86_64-pc-windows-msvc.zip
```

## Runtime profiles

The built-in profiles are deterministic and do not contain secrets:

- `Default`
- `LocalOnly`
- `Offline`
- `LowMemory`
- `Research`

Profiles control network availability, local-model preference, context limits, and local telemetry capacity.

## Diagnostics boundary

Diagnostics are local-only, bounded, and redact secret-like values before rendering. They must never include raw API keys, authorization headers, prompts, or model credentials.

## Release boundary

Phase 10 does not publish packages automatically and does not install unsigned binaries. Publishing, signing, notarization, Homebrew, MSI, and Linux package repositories remain explicit release-operator actions.
