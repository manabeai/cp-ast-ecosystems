# Development Conventions

## Language
- Rust 2021 edition, minimum version 1.75

## Build & Test
- Build: `cargo build`
- Test: `cargo test`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Format: `cargo fmt --all`
- Format check: `cargo fmt --all -- --check`

## Conventions
- All public types derive `Debug, Clone`
- Use `#[must_use]` on functions returning values
- Prefer `Result<T, E>` over panics
- No `unwrap()` in library code (tests OK)
- Follow clippy pedantic suggestions where reasonable

## Project Structure
- `crates/cp-ast-core/` — Core AST types (StructureAST + ConstraintAST)
- `crates/cp-ast-json/` — JSON bridge for AST serialization and browser/native exchange
- `crates/cp-ast-tree/` — ASCII tree renderer for AST inspection (depends on cp-ast-core)
- `crates/cp-ast-wasm/` — wasm-bindgen bridge for external web consumers
- `random-test-cli/` — related CLI submodule
- `doc/` — Design documents and references
- `doc/plan/main.md` — Project architecture overview (full vision)
- `doc/plan/processing.md` — Implementation progress and roadmap
- `doc/plan/repository-split.md` — current repository boundary and frontend split note
- `docs/superpowers/` — Specs and plans

## Repository Boundary
- This repo owns the Rust AST ecosystem and wasm bindings.
- The web frontend has been split out to `random-test-creator`.
- `random-test-creator` should consume this repository as a git submodule instead of keeping frontend code under `web/` here.

## Git
- Pre-commit hooks: `.githooks/pre-commit` (fmt + clippy)
- Configure: `git config core.hooksPath .githooks`

## Other
- call by execute command `copilot-session-end.sh` when yout task finished or you are waiting for user response
- When invoking subagents, explicitly specify using Claude Opus model
