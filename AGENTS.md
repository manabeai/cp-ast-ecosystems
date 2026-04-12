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
- `doc/` — Design documents and references
- `docs/superpowers/` — Specs and plans

## Git
- Pre-commit hooks: `.githooks/pre-commit` (fmt + clippy)
- Configure: `git config core.hooksPath .githooks`

## Other
- call by execute command `copilot-session-end.sh` when yout task finished or you are waiting for user response
