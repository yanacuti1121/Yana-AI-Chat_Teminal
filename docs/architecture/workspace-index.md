# Workspace Index Contract

The workspace index is a deterministic read-only discovery layer.

## Responsibilities

- enumerate bounded workspace files in stable order;
- index lightweight Rust symbols and dependency roots;
- maintain reverse dependency lookups;
- refresh only explicitly changed paths;
- infer project intent from manifest files;
- detect workspace capabilities from files that actually exist.

## Non-responsibilities

The index does not:

- call a language model;
- mutate workspace files;
- execute project commands;
- bypass `.gitignore` policy through symlinks;
- infer capabilities that have no file evidence;
- replace Atlas or Knowledge Engine persistence.

## Determinism

Directory entries, changed paths, symbols, and references are sorted before they are exposed. Stable FNV-1a content hashes decide whether an incremental refresh is necessary.

## Safety limits

Every full build is bounded by maximum file count and maximum file size. Symlinks are ignored. Incremental paths must be relative, cannot contain parent traversal, and must canonicalize inside the workspace root.

## Workspace intent

The detector recognizes Rust, Next.js, Node, Python, Go, mixed, and unknown workspaces from conventional manifests. Capability detection is evidence-based and currently includes tests, benchmarks, CI, Docker, documentation, release pipeline, and Git metadata.
