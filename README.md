[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/manabeai/cp-ast-ecosystems)

# cp-ast-ecosystems

`cp-ast-ecosystems` is the Rust-side source of truth for competitive-programming input ASTs.
It owns the core AST model, renderers, sample generation, JSON bridge, tree viewer, and wasm bindings.

## Repository scope

This repository now focuses on the Rust and wasm layers:

- `crates/cp-ast-core/` - core AST, operations, projection, renderers, sample generation
- `crates/cp-ast-json/` - JSON DTO bridge for AST exchange
- `crates/cp-ast-tree/` - ASCII tree renderer for inspection
- `crates/cp-ast-wasm/` - wasm-bindgen bridge for browser consumers
- `random-test-cli/` - related CLI kept as a git submodule

The web frontend is no longer developed in this repository.
It should live in the separate `random-test-creator` repository and consume `cp-ast-ecosystems` as a git submodule so tags for the frontend and the Rust AST stack can move independently.

See [doc/plan/repository-split.md](/home/mana/programs/cp-ast-ecosystems/doc/plan/repository-split.md) for the intended repo boundary.

## Build and test

```bash
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```
