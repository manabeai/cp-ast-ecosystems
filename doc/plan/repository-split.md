# Repository Split

## Current boundary

`cp-ast-ecosystems` now owns the reusable Rust stack:

- `cp-ast-core`
- `cp-ast-json`
- `cp-ast-tree`
- `cp-ast-wasm`

The browser application is intentionally out of scope for this repository.

## Frontend location

The web frontend should live in the separate `random-test-creator` repository.
That repository should consume `cp-ast-ecosystems` as a git submodule so:

- Rust AST tags can be cut independently
- frontend release cadence can move independently
- browser-specific dependencies do not affect this repository's release workflow

## Expected downstream shape

A downstream frontend repository is expected to:

1. keep its own `package.json`, Vite/Playwright setup, and release tags
2. pin `cp-ast-ecosystems` as a submodule
3. build `crates/cp-ast-wasm` from the submodule when it needs browser bindings

Historical design documents in this repository may still refer to the old in-tree `web/` layout.
Treat those references as implementation history, not the current repository structure.
