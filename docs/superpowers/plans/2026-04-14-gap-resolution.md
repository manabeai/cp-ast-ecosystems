# Gap Resolution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend Repeat and Array nodes to accept Expression-based counts/lengths, add loop variables to Repeat, and improve Choice rendering — resolving Gaps A, B, H, D from the AtCoder coverage survey.

**Architecture:** Change `Repeat.count` and `Array.length` from `Reference` to `Expression`, add `Repeat.index_var: Option<Ident>`, add `resolve_expression_as_int()` to the sample generator with loop variable scoping, and improve Choice rendering in both TeX and plain text renderers.

**Tech Stack:** Rust 2021, cp-ast-core crate

**Design spec:** `docs/superpowers/specs/2026-04-14-gap-resolution-design.md`

---

## File Map

| File | Responsibility | Tasks |
|------|---------------|-------|
| `src/structure/node_kind.rs` | Core NodeKind enum — Repeat + Array type changes | T-01 |
| `src/sample/dependency.rs` | Dependency graph — Expression reference extraction | T-02 |
| `src/sample/generator.rs` | Sample generation — expression resolution + loop vars | T-03, T-04 |
| `src/sample/output.rs` | Sample output — pattern match updates | T-03 |
| `src/render/mod.rs` | Plain text helpers — `render_expression()` | T-05 |
| `src/render/input_format.rs` | Plain text renderer — Repeat/Array/Choice rendering | T-05 |
| `src/render_tex/tex_helpers.rs` | TeX helpers — `expression_to_tex()` | T-06 |
| `src/render_tex/input_tex.rs` | TeX renderer — Repeat/Array/Choice rendering | T-06 |
| `src/operation/node_ops.rs` | Node operations — pattern match fixups | T-01 |
| `src/operation/multi_test_case.rs` | Multi-test-case wrapper — Expression wrap | T-01 |
| `src/operation/fill_hole.rs` | Fill hole — length_spec migration | T-01 |
| `src/projection/projection_impl.rs` | Projection API — pattern match fixups | T-01 |
| `tests/structure_ast.rs` | Structure unit tests — migration | T-01 |
| `tests/render_basic.rs` | Plain text render tests — migration | T-01 |
| `tests/render_tex_basic.rs` | TeX render tests — migration | T-01 |
| `tests/sample_basic.rs` | Sample generation tests — migration + new tests | T-01, T-03, T-04 |
| `tests/e2e_abc284c.rs` | End-to-end test — migration | T-01 |
| `tests/typical_problem.rs` | Typical problem test — migration | T-01 |
| `tests/operation_basic.rs` | Operation tests — migration | T-01 |
| `tests/gap_resolution.rs` | NEW — E2E tests for graph, triangular, query patterns | T-07 |

---

### Task T-01: Migrate NodeKind::Repeat and NodeKind::Array to use Expression

This is the foundation — change the type definitions and fix all compilation errors. No new behavior yet; just wrap existing References in `Expression::Var()`.

**Files:**
- Modify: `crates/cp-ast-core/src/structure/node_kind.rs:14,24`
- Modify: `crates/cp-ast-core/src/sample/dependency.rs:58-62,72-79`
- Modify: `crates/cp-ast-core/src/sample/generator.rs:288,316-318,325-328,376-381,502-508`
- Modify: `crates/cp-ast-core/src/sample/output.rs:88`
- Modify: `crates/cp-ast-core/src/render/input_format.rs:31-32,69`
- Modify: `crates/cp-ast-core/src/render_tex/input_tex.rs:81-83,107-109`
- Modify: `crates/cp-ast-core/src/render_tex/tex_helpers.rs:179`
- Modify: `crates/cp-ast-core/src/operation/node_ops.rs:127-138,208,261-267`
- Modify: `crates/cp-ast-core/src/operation/multi_test_case.rs:28,55-58`
- Modify: `crates/cp-ast-core/src/operation/fill_hole.rs:71-76,125-131`
- Modify: `crates/cp-ast-core/src/projection/projection_impl.rs:165,183,220`
- Modify: `crates/cp-ast-core/tests/structure_ast.rs`
- Modify: `crates/cp-ast-core/tests/render_basic.rs`
- Modify: `crates/cp-ast-core/tests/render_tex_basic.rs`
- Modify: `crates/cp-ast-core/tests/sample_basic.rs`
- Modify: `crates/cp-ast-core/tests/e2e_abc284c.rs`
- Modify: `crates/cp-ast-core/tests/typical_problem.rs`
- Modify: `crates/cp-ast-core/tests/operation_basic.rs`

- [ ] **Step 1: Change NodeKind::Repeat and NodeKind::Array definitions**

In `crates/cp-ast-core/src/structure/node_kind.rs`, add the Expression import and change the two variants:

```rust
use super::node_id::NodeId;
use super::reference::Reference;
use super::types::{Ident, Literal, NodeKindHint};
use crate::constraint::Expression;
```

Change `Array`:
```rust
    /// 1D array: `A_1` ... `A_N`.
    Array { name: Ident, length: Expression },
```

Change `Repeat`:
```rust
    /// Variable-dependent repetition: M lines, T test cases.
    Repeat {
        count: Expression,
        index_var: Option<Ident>,
        body: Vec<NodeId>,
    },
```

- [ ] **Step 2: Fix dependency.rs — extract variable refs from Expression**

In `crates/cp-ast-core/src/sample/dependency.rs`, add a helper function and update Array + Repeat arms:

```rust
use crate::constraint::Expression;
use crate::structure::{NodeId, Reference};

/// Recursively extract all `VariableRef` NodeIds from an Expression.
fn extract_var_refs(expr: &Expression) -> Vec<NodeId> {
    match expr {
        Expression::Lit(_) => vec![],
        Expression::Var(Reference::VariableRef(id)) => vec![*id],
        Expression::Var(_) => vec![],
        Expression::BinOp { lhs, rhs, .. } => {
            let mut refs = extract_var_refs(lhs);
            refs.extend(extract_var_refs(rhs));
            refs
        }
        Expression::Pow { base, exp } => {
            let mut refs = extract_var_refs(base);
            refs.extend(extract_var_refs(exp));
            refs
        }
        Expression::FnCall { args, .. } => args.iter().flat_map(extract_var_refs).collect(),
    }
}
```

Update the Array arm (was lines 58-62):
```rust
                NodeKind::Array { length, .. } => {
                    for ref_id in extract_var_refs(length) {
                        deps.entry(id).or_default().push(ref_id);
                    }
                }
```

Update the Repeat arm (was lines 72-79):
```rust
                NodeKind::Repeat { count, body, .. } => {
                    for ref_id in extract_var_refs(count) {
                        deps.entry(id).or_default().push(ref_id);
                    }
                    for &child in body {
                        deps.entry(child).or_default().push(id);
                    }
                }
```

- [ ] **Step 3: Fix generator.rs — temporary shim for expression resolution**

For this task only, add a temporary shim that extracts the inner Reference from `Expression::Var` and delegates to `resolve_reference_as_int`. The full `resolve_expression_as_int` comes in T-03.

In `crates/cp-ast-core/src/sample/generator.rs`:

Add a temporary helper method to `GenerationContext`:
```rust
    /// Temporary shim: resolve Expression by extracting inner Var reference.
    /// Full expression evaluation added in T-03.
    fn resolve_expression_as_int_shim(
        &self,
        expr: &Expression,
    ) -> Result<i64, GenerationError> {
        match expr {
            Expression::Var(reference) => self.resolve_reference_as_int(reference),
            Expression::Lit(v) => Ok(*v),
            _ => Err(GenerationError::InvalidExpression(
                "complex expressions not yet supported".into(),
            )),
        }
    }
```

Update the skip-set arm (line ~288):
```rust
                NodeKind::Repeat { body, .. } => {
                    for &child in body {
                        skip.insert(child);
                    }
                }
```

Update generate_node_inner Array arm (line ~316):
```rust
            NodeKind::Array { length, .. } => {
                let length = length.clone();
                self.generate_array(node_id, &length)
            }
```

Update generate_node_inner Repeat arm (line ~325):
```rust
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                let count = count.clone();
                let index_var = index_var.clone();
                let body = body.clone();
                self.generate_repeat(node_id, &count, &index_var, &body)
            }
```

Change `generate_array` signature and body (line ~376):
```rust
    fn generate_array(
        &mut self,
        node_id: NodeId,
        length_expr: &Expression,
    ) -> Result<(), GenerationError> {
        let len = self.resolve_expression_as_int_shim(length_expr)?;
        // ... rest unchanged
```

Change `generate_repeat` signature and body (line ~502):
```rust
    fn generate_repeat(
        &mut self,
        node_id: NodeId,
        count_expr: &Expression,
        _index_var: &Option<Ident>,
        body: &[NodeId],
    ) -> Result<(), GenerationError> {
        let count = self.resolve_expression_as_int_shim(count_expr)?;
        // ... rest unchanged
```

- [ ] **Step 4: Fix output.rs — update Repeat pattern match**

In `crates/cp-ast-core/src/sample/output.rs` (line ~88), update the pattern:
```rust
        NodeKind::Repeat { body, .. } => {
```
(Using `..` already covers the new `index_var` field. Verify that both occurrences use `..`.)

- [ ] **Step 5: Fix render/input_format.rs — Array and Repeat arms**

In `crates/cp-ast-core/src/render/input_format.rs`:

Add import at top:
```rust
use crate::constraint::Expression;
```

Update Array arm (line 31) — temporarily extract inner reference:
```rust
        NodeKind::Array { name, length } => {
            let length_str = match length {
                Expression::Var(r) => render_reference(engine, r),
                _ => format!("{length:?}"),
            };
            writeln!(
                output,
                "{}_1 {}_2 … {}_{}",
                name.as_str(),
                name.as_str(),
                name.as_str(),
                length_str
            )
            .unwrap();
        }
```

Repeat arm (line 69) — no content change needed, `count: _` pattern still works:
```rust
        NodeKind::Repeat { count: _, body, .. } => {
```

Choice arm (line 115) — no change yet (improved in T-05).

- [ ] **Step 6: Fix render_tex/input_tex.rs — Array and Repeat arms**

In `crates/cp-ast-core/src/render_tex/input_tex.rs`:

Add import:
```rust
use crate::constraint::Expression;
```

Update Array arm (line 81) — extract inner reference temporarily:
```rust
        NodeKind::Array { name, length } => {
            let name_str = ident_to_tex(name);
            let length_str = match length {
                Expression::Var(r) => reference_to_tex(engine, r, warnings),
                _ => format!("{length:?}"),
            };
            lines.push(format!(
                "{name_str}_1 \\ {name_str}_2 \\ \\cdots \\ {name_str}_{length_str}"
            ));
        }
```

Update Repeat arm (line 107):
```rust
        NodeKind::Repeat { count, body, .. } => {
            let count_str = match count {
                Expression::Var(r) => reference_to_tex(engine, r, warnings),
                _ => format!("{count:?}"),
            };
            render_repeat_lines(engine, &count_str, body, lines, warnings, options);
        }
```

- [ ] **Step 7: Fix render_tex/tex_helpers.rs — resolve_array_info**

In `crates/cp-ast-core/src/render_tex/tex_helpers.rs` (line 179):

Add import:
```rust
use crate::constraint::Expression;
```

Update `resolve_array_info` return type and body:
```rust
pub fn resolve_array_info(
    engine: &AstEngine,
    reference: &Reference,
) -> Option<(String, Expression)> {
    if let Reference::VariableRef(node_id) = reference {
        if let Some(node) = engine.structure.get(*node_id) {
            if let NodeKind::Array { name, length } = node.kind() {
                return Some((ident_to_tex(name), length.clone()));
            }
        }
    }
    None
}
```

Find all callers of `resolve_array_info` and update them to handle `Expression` instead of `Reference`. The `length` is used in format strings via `reference_to_tex` — temporarily wrap:
```rust
// At the call site, extract the reference if it's Expression::Var
if let Some((name, length_expr)) = resolve_array_info(engine, reference) {
    let length_str = match &length_expr {
        Expression::Var(r) => reference_to_tex(engine, r, warnings),
        _ => format!("{length_expr:?}"),
    };
    // ... use length_str
}
```

- [ ] **Step 8: Fix operation/node_ops.rs — 3 Repeat pattern matches**

In `crates/cp-ast-core/src/operation/node_ops.rs`:

Add import:
```rust
use crate::constraint::Expression;
```

Update `add_slot_element` Repeat arm (line ~127):
```rust
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                if slot_name == "body" {
                    let mut new_body = body.clone();
                    new_body.push(new_node_id);
                    parent_node.set_kind(NodeKind::Repeat {
                        count: count.clone(),
                        index_var: index_var.clone(),
                        body: new_body,
                    });
```

Update `remove_slot_element` check arm (line ~208):
```rust
            NodeKind::Section { body, .. } | NodeKind::Repeat { body, .. } => {
```
(Already uses `..` — no change needed here.)

Update `remove_slot_element` mutation arm (line ~261):
```rust
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                let mut new_body = body.clone();
                new_body.retain(|&id| id != child);
                parent_node.set_kind(NodeKind::Repeat {
                    count: count.clone(),
                    index_var: index_var.clone(),
                    body: new_body,
                });
            }
```

- [ ] **Step 9: Fix operation/multi_test_case.rs — Repeat creation**

In `crates/cp-ast-core/src/operation/multi_test_case.rs` (line ~55):

Add import:
```rust
use crate::constraint::Expression;
```

```rust
        let repeat_id = self.structure.add_node(NodeKind::Repeat {
            count: Expression::Var(Reference::VariableRef(count_scalar_id)),
            index_var: None,
            body: current_children,
        });
```

- [ ] **Step 10: Fix operation/fill_hole.rs — length_spec_to_reference**

In `crates/cp-ast-core/src/operation/fill_hole.rs`:

Add import:
```rust
use crate::constraint::Expression;
```

Change `length_spec_to_reference` to return `Expression`:
```rust
fn length_spec_to_expression(spec: &LengthSpec) -> Expression {
    match spec {
        LengthSpec::Fixed(n) => Expression::Var(Reference::Unresolved(Ident::new(&format!("{n}")))),
        LengthSpec::RefVar(id) => Expression::Var(Reference::VariableRef(*id)),
        LengthSpec::Expr(s) => Expression::Var(Reference::Unresolved(Ident::new(s))),
    }
}
```

Update the Array creation call site (line ~72):
```rust
                FillContent::Array { name, length, .. } => {
                    let length_expr = length_spec_to_expression(length);
                    NodeKind::Array {
                        name: Ident::new(name),
                        length: length_expr,
                    }
                }
```

- [ ] **Step 11: Fix projection/projection_impl.rs — 3 pattern matches**

In `crates/cp-ast-core/src/projection/projection_impl.rs`, the existing patterns already use `..` for Repeat and named fields for Array. Update:

Line 165 — no change needed (uses `NodeKind::Array { name, .. }`).

Line 183 — no change needed (uses `NodeKind::Repeat { body, .. }`).

Line 220 — no change needed (uses `NodeKind::Repeat { body, .. }`).

Verify no compilation errors from the `index_var` addition.

- [ ] **Step 12: Migrate all test files**

For every test file, wrap `Reference` values in `Expression::Var()` and add `index_var: None` to Repeat nodes. Add `use cp_ast_core::constraint::Expression;` to imports.

**Pattern for Array:**
```rust
// Before:
NodeKind::Array { name: Ident::new("A"), length: Reference::VariableRef(n_id) }
// After:
NodeKind::Array { name: Ident::new("A"), length: Expression::Var(Reference::VariableRef(n_id)) }
```

**Pattern for Repeat:**
```rust
// Before:
NodeKind::Repeat { count: Reference::VariableRef(n_id), body: vec![...] }
// After:
NodeKind::Repeat { count: Expression::Var(Reference::VariableRef(n_id)), index_var: None, body: vec![...] }
```

Files to update (count of Array/Repeat occurrences):
- `tests/structure_ast.rs` — 2 (1 Array, 1 Repeat)
- `tests/render_basic.rs` — 2 (1 Array, 1 Repeat)
- `tests/render_tex_basic.rs` — 8 (5 Array, 3 Repeat)
- `tests/sample_basic.rs` — 18 (8 Array, 10 Repeat)
- `tests/e2e_abc284c.rs` — 1 (1 Repeat)
- `tests/typical_problem.rs` — 1 (1 Array)
- `tests/operation_basic.rs` — 3 (2 Array via matches!, 1 Repeat)

- [ ] **Step 13: Run full build + test**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

Expected: All 192 tests pass, no clippy warnings, no fmt issues. Behavior is identical to before — this is a pure refactor.

- [ ] **Step 14: Commit**

```bash
git add -A
git commit -m "refactor: migrate Repeat.count and Array.length to Expression

- Repeat.count: Reference → Expression (Gap A foundation)
- Array.length: Reference → Expression (Gap D foundation)
- Repeat gains index_var: Option<Ident> (Gap H foundation)
- All existing code wraps References in Expression::Var()
- Temporary resolve_expression_as_int_shim() for backwards compat
- No behavioral changes; all 192 tests pass

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task T-02: Implement resolve_expression_as_int with loop variable support

Replace the temporary shim with full Expression resolution and loop variable scoping.

**Files:**
- Modify: `crates/cp-ast-core/src/sample/generator.rs`
- Test: `crates/cp-ast-core/tests/sample_basic.rs`

- [ ] **Step 1: Write failing tests for Expression-based count**

Append to `crates/cp-ast-core/tests/sample_basic.rs`:

```rust
#[test]
fn repeat_with_expression_count_n_minus_1() {
    // Graph pattern: N nodes, N-1 edges (tree)
    use cp_ast_core::constraint::{ArithOp, Expression};

    let mut engine = AstEngine::default();

    // N = 5
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: n_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: n_id,
        lo: Some(Expression::Lit(5)),
        hi: Some(Expression::Lit(5)),
    });

    // u_i, v_i scalars
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: u_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: u_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Var(Reference::VariableRef(n_id))),
    });

    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: v_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: v_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Var(Reference::VariableRef(n_id))),
    });

    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });

    // Repeat N-1 times
    let count_expr = Expression::BinOp {
        op: ArithOp::Sub,
        lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
        rhs: Box::new(Expression::Lit(1)),
    };
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: count_expr,
        index_var: None,
        body: vec![tuple_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    // N=5, so N-1=4 edges
    assert_eq!(sample.repeat_instances[&repeat_id].len(), 4);

    let output = render_sample(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 5); // 1 line for N + 4 lines for edges
    assert_eq!(lines[0], "5");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test sample_basic repeat_with_expression_count_n_minus_1 -- --nocapture`

Expected: FAIL — `resolve_expression_as_int_shim` returns error for BinOp.

- [ ] **Step 3: Implement full resolve_expression_as_int**

In `crates/cp-ast-core/src/sample/generator.rs`, add `loop_vars` field and replace the shim:

Add the field to `GenerationContext`:
```rust
struct GenerationContext<'a> {
    engine: &'a AstEngine,
    rng: StdRng,
    values: HashMap<NodeId, SampleValue>,
    repeat_instances: HashMap<NodeId, Vec<HashMap<NodeId, SampleValue>>>,
    config: GenerationConfig,
    loop_vars: HashMap<Ident, i64>,
}
```

Initialize in the constructor:
```rust
        let mut ctx = GenerationContext {
            engine,
            rng: StdRng::seed_from_u64(seed),
            values: HashMap::new(),
            repeat_instances: HashMap::new(),
            config,
            loop_vars: HashMap::new(),
        };
```

Replace `resolve_expression_as_int_shim` with:
```rust
    fn resolve_expression_as_int(
        &self,
        expr: &Expression,
    ) -> Result<i64, GenerationError> {
        match expr {
            Expression::Lit(v) => Ok(*v),
            Expression::Var(reference) => {
                // Check loop variables first for Unresolved references
                if let Reference::Unresolved(name) = reference {
                    if let Some(&val) = self.loop_vars.get(name) {
                        return Ok(val);
                    }
                }
                self.resolve_reference_as_int(reference)
            }
            Expression::BinOp { op, lhs, rhs } => {
                let l = self.resolve_expression_as_int(lhs)?;
                let r = self.resolve_expression_as_int(rhs)?;
                match op {
                    ArithOp::Add => l.checked_add(r),
                    ArithOp::Sub => l.checked_sub(r),
                    ArithOp::Mul => l.checked_mul(r),
                    ArithOp::Div => {
                        if r == 0 {
                            None
                        } else {
                            l.checked_div(r)
                        }
                    }
                }
                .ok_or_else(|| {
                    GenerationError::InvalidExpression("arithmetic overflow".into())
                })
            }
            Expression::Pow { base, exp } => {
                let b = self.resolve_expression_as_int(base)?;
                let e = self.resolve_expression_as_int(exp)?;
                let e_u32 = u32::try_from(e).map_err(|_| {
                    GenerationError::InvalidExpression("negative exponent".into())
                })?;
                b.checked_pow(e_u32).ok_or_else(|| {
                    GenerationError::InvalidExpression("power overflow".into())
                })
            }
            Expression::FnCall { name, args } => {
                let resolved: Result<Vec<i64>, _> = args
                    .iter()
                    .map(|a| self.resolve_expression_as_int(a))
                    .collect();
                let resolved = resolved?;
                match name.as_str() {
                    "min" => resolved
                        .iter()
                        .copied()
                        .min()
                        .ok_or_else(|| {
                            GenerationError::InvalidExpression("min() with no args".into())
                        }),
                    "max" => resolved
                        .iter()
                        .copied()
                        .max()
                        .ok_or_else(|| {
                            GenerationError::InvalidExpression("max() with no args".into())
                        }),
                    other => Err(GenerationError::InvalidExpression(
                        format!("unsupported function: {other}"),
                    )),
                }
            }
        }
    }
```

Update `generate_array` to call `resolve_expression_as_int`:
```rust
    fn generate_array(
        &mut self,
        node_id: NodeId,
        length_expr: &Expression,
    ) -> Result<(), GenerationError> {
        let len = self.resolve_expression_as_int(length_expr)?;
```

Update `generate_repeat` to call `resolve_expression_as_int`:
```rust
    fn generate_repeat(
        &mut self,
        node_id: NodeId,
        count_expr: &Expression,
        _index_var: &Option<Ident>,
        body: &[NodeId],
    ) -> Result<(), GenerationError> {
        let count = self.resolve_expression_as_int(count_expr)?;
```

Remove `resolve_expression_as_int_shim`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test sample_basic repeat_with_expression_count_n_minus_1 -- --nocapture`

Expected: PASS — Expression `N-1` resolves to 4, generates 4 iterations.

- [ ] **Step 5: Write test for literal Expression count**

Append to `crates/cp-ast-core/tests/sample_basic.rs`:

```rust
#[test]
fn repeat_with_literal_count() {
    use cp_ast_core::constraint::Expression;

    let mut engine = AstEngine::default();
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: x_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: x_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(100)),
    });

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Lit(3),
        index_var: None,
        body: vec![x_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    assert_eq!(sample.repeat_instances[&repeat_id].len(), 3);
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test --test sample_basic repeat_with_literal_count -- --nocapture`

Expected: PASS.

- [ ] **Step 7: Run full test suite**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

Expected: All tests pass (192 existing + 2 new = 194).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(sample): implement full Expression resolution for Repeat/Array counts

- resolve_expression_as_int() handles Lit, Var, BinOp, Pow, FnCall
- loop_vars HashMap added to GenerationContext (populated in T-03)
- Supports N-1 style graph patterns (Gap A)
- Remove temporary resolve_expression_as_int_shim

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task T-03: Implement loop variable support in generate_repeat

Add loop variable scoping so body nodes can reference the iteration index.

**Files:**
- Modify: `crates/cp-ast-core/src/sample/generator.rs`
- Test: `crates/cp-ast-core/tests/sample_basic.rs`

- [ ] **Step 1: Write failing test for loop variable**

Append to `crates/cp-ast-core/tests/sample_basic.rs`:

```rust
#[test]
fn repeat_with_loop_variable_basic() {
    // Triangular pattern: row i has (i+1) elements
    use cp_ast_core::constraint::{ArithOp, Expression};

    let mut engine = AstEngine::default();

    // N = 3
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: n_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: n_id,
        lo: Some(Expression::Lit(3)),
        hi: Some(Expression::Lit(3)),
    });

    // Array C with length = i + 1 (loop variable)
    let c_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("C"),
        length: Expression::BinOp {
            op: ArithOp::Add,
            lhs: Box::new(Expression::Var(Reference::Unresolved(Ident::new("i")))),
            rhs: Box::new(Expression::Lit(1)),
        },
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: c_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: c_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(9)),
    });

    // Repeat N times with index_var "i"
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: Some(Ident::new("i")),
        body: vec![c_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();

    // 3 iterations
    let instances = &sample.repeat_instances[&repeat_id];
    assert_eq!(instances.len(), 3);

    // Row 0: i=0, length = 0+1 = 1 element
    // Row 1: i=1, length = 1+1 = 2 elements
    // Row 2: i=2, length = 2+1 = 3 elements
    for (i, iteration) in instances.iter().enumerate() {
        if let Some(SampleValue::Array(elements)) = iteration.get(&c_id) {
            assert_eq!(
                elements.len(),
                i + 1,
                "row {i} should have {} elements, got {}",
                i + 1,
                elements.len()
            );
        } else {
            panic!("row {i} missing array value for c_id");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test sample_basic repeat_with_loop_variable_basic -- --nocapture`

Expected: FAIL — loop variable "i" is not resolved (Unresolved reference error).

- [ ] **Step 3: Implement loop variable scoping in generate_repeat**

In `crates/cp-ast-core/src/sample/generator.rs`, update `generate_repeat`:

```rust
    fn generate_repeat(
        &mut self,
        node_id: NodeId,
        count_expr: &Expression,
        index_var: &Option<Ident>,
        body: &[NodeId],
    ) -> Result<(), GenerationError> {
        let count = self.resolve_expression_as_int(count_expr)?;
        let count_usize = usize::try_from(count)
            .map_err(|_| GenerationError::InvalidExpression("negative repeat count".into()))?;

        if count_usize > self.config.max_repeat_count {
            return Err(GenerationError::InvalidStructure(format!(
                "repeat count {count_usize} exceeds limit {}",
                self.config.max_repeat_count
            )));
        }

        let mut instances = Vec::with_capacity(count_usize);

        for i in 0..count_usize {
            // Set loop variable if present
            if let Some(var_name) = index_var {
                self.loop_vars.insert(
                    var_name.clone(),
                    i64::try_from(i).unwrap_or(i as i64),
                );
            }

            // Generate body children
            for &child_id in body {
                if let Some(node) = self.engine.structure.get(child_id) {
                    let kind = node.kind().clone();
                    self.generate_node_inner(child_id, &kind)?;
                }
            }

            // Snapshot iteration values
            let mut iteration_values = HashMap::new();
            for &child_id in body {
                if let Some(val) = self.values.get(&child_id) {
                    iteration_values.insert(child_id, val.clone());
                }
            }
            instances.push(iteration_values);

            // Remove body child values for next iteration
            for &child_id in body {
                self.values.remove(&child_id);
            }
        }

        // Remove loop variable after loop
        if let Some(var_name) = index_var {
            self.loop_vars.remove(var_name);
        }

        self.repeat_instances.insert(node_id, instances);
        Ok(())
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test sample_basic repeat_with_loop_variable_basic -- --nocapture`

Expected: PASS — loop variable "i" resolves to 0, 1, 2 giving array lengths 1, 2, 3.

- [ ] **Step 5: Write test for triangular matrix output**

Append to `crates/cp-ast-core/tests/sample_basic.rs`:

```rust
#[test]
fn repeat_with_loop_variable_output() {
    use cp_ast_core::constraint::{ArithOp, Expression};

    let mut engine = AstEngine::default();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: n_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: n_id,
        lo: Some(Expression::Lit(3)),
        hi: Some(Expression::Lit(3)),
    });

    // Array with length = N - i (decreasing)
    let c_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("C"),
        length: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Var(Reference::Unresolved(Ident::new("i")))),
        },
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: c_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: c_id,
        lo: Some(Expression::Lit(0)),
        hi: Some(Expression::Lit(9)),
    });

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(n_id)),
        index_var: Some(Ident::new("i")),
        body: vec![c_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let output = render_sample(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();

    // N=3, rows have 3, 2, 1 elements
    assert_eq!(lines.len(), 4); // N + 3 rows
    assert_eq!(lines[0], "3");
    assert_eq!(lines[1].split_whitespace().count(), 3); // i=0: N-0=3
    assert_eq!(lines[2].split_whitespace().count(), 2); // i=1: N-1=2
    assert_eq!(lines[3].split_whitespace().count(), 1); // i=2: N-2=1
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test --test sample_basic repeat_with_loop_variable_output -- --nocapture`

Expected: PASS.

- [ ] **Step 7: Run full test suite**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

Expected: All tests pass (194 + 2 = 196).

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(sample): add loop variable support to Repeat generation

- generate_repeat() sets loop_vars[index_var] = i during iteration
- resolve_expression_as_int() resolves Unresolved refs from loop_vars
- Enables triangular/jagged arrays (Gap H + Gap D)
- Tests: loop variable basic + decreasing-length output

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task T-04: Verify and test Choice in Repeat

Confirm that Choice nodes inside Repeat bodies work correctly in sample generation.

**Files:**
- Test: `crates/cp-ast-core/tests/sample_basic.rs`

- [ ] **Step 1: Write test for Choice in Repeat**

Append to `crates/cp-ast-core/tests/sample_basic.rs`:

```rust
#[test]
fn choice_in_repeat_generates_independently() {
    // Query pattern: Q queries, each with tag T choosing variant
    use cp_ast_core::constraint::Expression;

    let mut engine = AstEngine::default();

    // Q = 10
    let q_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: q_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: q_id,
        lo: Some(Expression::Lit(10)),
        hi: Some(Expression::Lit(10)),
    });

    // Tag T (will be set by Choice)
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });

    // Variant 1: single X scalar
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: x_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: x_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(100)),
    });

    // Variant 2: two scalars Y, Z
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: y_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: y_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(100)),
    });

    let z_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Z"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: z_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: z_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(100)),
    });

    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id, z_id]),
        ],
    });

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(q_id)),
        index_var: None,
        body: vec![choice_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![q_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let instances = &sample.repeat_instances[&repeat_id];
    assert_eq!(instances.len(), 10);

    // Check that at least one variant 1 and one variant 2 were chosen
    // (with 10 iterations and seed 42, extremely likely)
    let mut saw_variant1 = false;
    let mut saw_variant2 = false;
    for iteration in instances {
        if let Some(SampleValue::Int(tag)) = iteration.get(&t_id) {
            match tag {
                1 => saw_variant1 = true,
                2 => saw_variant2 = true,
                _ => panic!("unexpected tag value: {tag}"),
            }
        }
    }
    assert!(saw_variant1, "should see at least one variant 1");
    assert!(saw_variant2, "should see at least one variant 2");

    let output = render_sample(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();
    // First line is Q=10, then 10 query lines
    assert_eq!(lines[0], "10");
    assert_eq!(lines.len(), 11);
}
```

- [ ] **Step 2: Run test**

Run: `cargo test --test sample_basic choice_in_repeat_generates_independently -- --nocapture`

Expected: PASS (Choice in Repeat already works structurally).

If it fails, investigate the `generate_repeat` snapshot mechanism — ensure Choice children (tag + variant children) are captured in `iteration_values`. The fix would be to walk Choice children recursively when snapshotting.

- [ ] **Step 3: Run full test suite**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

Expected: 197 tests pass.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test(sample): verify Choice in Repeat generates independently

- 10-iteration query pattern with 2 variants
- Confirms tag selection varies across iterations
- Validates Gap B (Choice in Repeat) works correctly

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task T-05: Improve plain text rendering for Expression and Choice

Add `render_expression()` helper and improve Choice rendering in plain text.

**Files:**
- Modify: `crates/cp-ast-core/src/render/mod.rs`
- Modify: `crates/cp-ast-core/src/render/input_format.rs`
- Test: `crates/cp-ast-core/tests/render_basic.rs`

- [ ] **Step 1: Write failing tests**

Append to `crates/cp-ast-core/tests/render_basic.rs`:

```rust
#[test]
fn render_expression_count_repeat() {
    use cp_ast_core::constraint::{ArithOp, Expression};

    let mut engine = AstEngine::default();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Lit(1)),
        },
        index_var: None,
        body: vec![tuple_id],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let text = render_input(&engine);
    // Should show indexed form for the repeat body
    assert!(text.contains("u_i v_i"), "got: {text}");
}

#[test]
fn render_choice_plain_text() {
    use cp_ast_core::constraint::Expression;

    let mut engine = AstEngine::default();
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let z_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Z"),
    });
    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id, z_id]),
        ],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![choice_id],
        });

    let text = render_input(&engine);
    assert!(text.contains("If T = 1: X"), "got: {text}");
    assert!(text.contains("If T = 2: Y Z"), "got: {text}");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test render_basic render_expression_count_repeat render_choice_plain_text -- --nocapture`

Expected: `render_choice_plain_text` FAIL (currently outputs `(choice)`). `render_expression_count_repeat` may pass if the existing body tuple handling works.

- [ ] **Step 3: Add render_expression to render/mod.rs**

In `crates/cp-ast-core/src/render/mod.rs`, add:

```rust
use crate::constraint::{ArithOp, Expression};

/// Render an `Expression` to a human-readable string.
pub(crate) fn render_expression(engine: &AstEngine, expr: &Expression) -> String {
    match expr {
        Expression::Lit(v) => v.to_string(),
        Expression::Var(reference) => render_reference(engine, reference),
        Expression::BinOp { op, lhs, rhs } => {
            let l = render_expression(engine, lhs);
            let r = render_expression(engine, rhs);
            let op_str = match op {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "*",
                ArithOp::Div => "/",
            };
            format!("{l}{op_str}{r}")
        }
        Expression::Pow { base, exp } => {
            let b = render_expression(engine, base);
            let e = render_expression(engine, exp);
            format!("{b}^{e}")
        }
        Expression::FnCall { name, args } => {
            let args_str: Vec<String> =
                args.iter().map(|a| render_expression(engine, a)).collect();
            format!("{}({})", name.as_str(), args_str.join(", "))
        }
    }
}
```

- [ ] **Step 4: Update input_format.rs — Array, Repeat, Choice rendering**

In `crates/cp-ast-core/src/render/input_format.rs`:

Update imports:
```rust
use super::{render_expression, render_reference};
```

Update Array arm:
```rust
        NodeKind::Array { name, length } => {
            let length_str = render_expression(engine, length);
            writeln!(
                output,
                "{}_1 {}_2 … {}_{}",
                name.as_str(),
                name.as_str(),
                name.as_str(),
                length_str
            )
            .unwrap();
        }
```

Update Choice arm (replace the placeholder):
```rust
        NodeKind::Choice { tag, variants } => {
            let tag_str = render_reference(engine, tag);
            for (literal, children) in variants {
                let lit_str = match literal {
                    Literal::IntLit(v) => v.to_string(),
                    Literal::StrLit(s) => format!("\"{s}\""),
                };
                let mut child_names = Vec::new();
                for &child_id in children {
                    if let Some(child_node) = engine.structure.get(child_id) {
                        if let NodeKind::Scalar { name } = child_node.kind() {
                            child_names.push(name.as_str().to_string());
                        } else {
                            child_names.push("<?>".to_string());
                        }
                    }
                }
                writeln!(output, "If {} = {}: {}", tag_str, lit_str, child_names.join(" "))
                    .unwrap();
            }
        }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test render_basic render_expression_count_repeat render_choice_plain_text -- --nocapture`

Expected: PASS.

- [ ] **Step 6: Run full test suite**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

Expected: All tests pass (199).

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(render): add Expression rendering and improve Choice plain text

- render_expression() converts Expression tree to human-readable text
- Choice now renders as 'If T = k: X Y Z' instead of '(choice)'
- Array.length rendered via render_expression()

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task T-06: Improve TeX rendering for Expression and Choice

Add `expression_to_tex()` and render Choice with `\begin{cases}` environment.

**Files:**
- Modify: `crates/cp-ast-core/src/render_tex/tex_helpers.rs`
- Modify: `crates/cp-ast-core/src/render_tex/input_tex.rs`
- Test: `crates/cp-ast-core/tests/render_tex_basic.rs`

- [ ] **Step 1: Write failing tests**

Append to `crates/cp-ast-core/tests/render_tex_basic.rs`:

```rust
#[test]
fn tex_expression_count_repeat() {
    use cp_ast_core::constraint::{ArithOp, Expression};

    let mut engine = AstEngine::default();
    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });
    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Lit(1)),
        },
        index_var: None,
        body: vec![tuple_id],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let result = render_input_tex(&engine);
    // Count should render as "N-1"
    assert!(
        result.tex.contains("N-1"),
        "should contain N-1, got: {}",
        result.tex
    );
}

#[test]
fn tex_choice_cases_environment() {
    use cp_ast_core::constraint::Expression;

    let mut engine = AstEngine::default();
    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });
    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id]),
        ],
    });
    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![choice_id],
        });

    let result = render_input_tex(&engine);
    assert!(
        result.tex.contains("\\begin{cases}"),
        "should use cases environment, got: {}",
        result.tex
    );
    assert!(
        result.tex.contains("\\text{if }"),
        "should have 'if' labels, got: {}",
        result.tex
    );
    assert!(
        result.tex.contains("\\end{cases}"),
        "should close cases environment, got: {}",
        result.tex
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test render_tex_basic tex_expression_count_repeat tex_choice_cases_environment -- --nocapture`

Expected: Both FAIL — current code uses `reference_to_tex` / `\texttt{(choice)}`.

- [ ] **Step 3: Add expression_to_tex to tex_helpers.rs**

In `crates/cp-ast-core/src/render_tex/tex_helpers.rs`, add:

```rust
/// Render an Expression to TeX notation.
pub fn expression_to_tex(
    engine: &AstEngine,
    expr: &Expression,
    warnings: &mut Vec<TexWarning>,
) -> String {
    match expr {
        Expression::Lit(v) => v.to_string(),
        Expression::Var(reference) => reference_to_tex(engine, reference, warnings),
        Expression::BinOp { op, lhs, rhs } => {
            let l = expression_to_tex(engine, lhs, warnings);
            let r = expression_to_tex(engine, rhs, warnings);
            let op_str = match op {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => " \\times ",
                ArithOp::Div => "/",
            };
            format!("{l}{op_str}{r}")
        }
        Expression::Pow { base, exp } => {
            let b = expression_to_tex(engine, base, warnings);
            let e = expression_to_tex(engine, exp, warnings);
            format!("{b}^{{{e}}}")
        }
        Expression::FnCall { name, args } => {
            let args_str: Vec<String> = args
                .iter()
                .map(|a| expression_to_tex(engine, a, warnings))
                .collect();
            format!(
                "\\mathrm{{{}}}({})",
                name.as_str(),
                args_str.join(", ")
            )
        }
    }
}
```

Make it `pub` so `input_tex.rs` can use it. Also update `resolve_array_info` to handle Expression length:

```rust
pub fn resolve_array_info(
    engine: &AstEngine,
    reference: &Reference,
    warnings: &mut Vec<TexWarning>,
) -> Option<(String, String)> {
    if let Reference::VariableRef(node_id) = reference {
        if let Some(node) = engine.structure.get(*node_id) {
            if let NodeKind::Array { name, length } = node.kind() {
                let length_str = expression_to_tex(engine, length, warnings);
                return Some((ident_to_tex(name), length_str));
            }
        }
    }
    None
}
```

Update all callers of `resolve_array_info` to use the new signature (returns `(String, String)` instead of `(String, Reference)`). The callers should pass `warnings` and get back a pre-rendered length string.

- [ ] **Step 4: Update input_tex.rs — Array, Repeat, Choice rendering**

In `crates/cp-ast-core/src/render_tex/input_tex.rs`:

Update Array arm:
```rust
        NodeKind::Array { name, length } => {
            let name_str = ident_to_tex(name);
            let length_str = expression_to_tex(engine, length, warnings);
            lines.push(format!(
                "{name_str}_1 \\ {name_str}_2 \\ \\cdots \\ {name_str}_{{{length_str}}}"
            ));
        }
```

Update Repeat arm:
```rust
        NodeKind::Repeat { count, body, .. } => {
            let count_str = expression_to_tex(engine, count, warnings);
            render_repeat_lines(engine, &count_str, body, lines, warnings, options);
        }
```

Replace Choice arm:
```rust
        NodeKind::Choice { tag, variants } => {
            let tag_str = reference_to_tex(engine, tag, warnings);
            lines.push("\\begin{cases}".to_owned());
            for (i, (literal, children)) in variants.iter().enumerate() {
                let mut variant_lines = Vec::new();
                for &child_id in children {
                    collect_lines(engine, child_id, &mut variant_lines, warnings, options);
                }
                let body_str = variant_lines.join(" \\ ");
                let lit_str = match literal {
                    Literal::IntLit(v) => v.to_string(),
                    Literal::StrLit(s) => format!("\\text{{{s}}}"),
                };
                let separator = if i + 1 < variants.len() {
                    " \\\\"
                } else {
                    ""
                };
                lines.push(format!(
                    "{body_str} & (\\text{{if }} {tag_str} = {lit_str}){separator}"
                ));
            }
            lines.push("\\end{cases}".to_owned());
        }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test render_tex_basic tex_expression_count_repeat tex_choice_cases_environment -- --nocapture`

Expected: PASS.

- [ ] **Step 6: Run full test suite**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

Expected: All tests pass (201).

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(render_tex): add expression_to_tex and Choice cases environment

- expression_to_tex() renders Expression tree as TeX math
- Choice now renders with \\begin{cases}...\\end{cases}
- Repeat count rendered via expression_to_tex() (shows N-1, etc.)
- Array length rendered via expression_to_tex()

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task T-07: End-to-end integration tests for gap patterns

Comprehensive tests for the three main patterns: graph input, triangular matrix, query branching.

**Files:**
- Create: `crates/cp-ast-core/tests/gap_resolution.rs`

- [ ] **Step 1: Create integration test file with graph pattern test**

Create `crates/cp-ast-core/tests/gap_resolution.rs`:

```rust
//! End-to-end integration tests for Gap Resolution (A, B, H, D).
//!
//! Tests real-world competitive programming input patterns that were
//! previously impossible to express with the AST.

use cp_ast_core::constraint::{ArithOp, Constraint, ExpectedType, Expression};
use cp_ast_core::operation::AstEngine;
use cp_ast_core::render::render_input;
use cp_ast_core::render_tex::render_input_tex;
use cp_ast_core::sample::generator::generate;
use cp_ast_core::sample::output::render_sample;
use cp_ast_core::structure::{Ident, Literal, NodeKind, Reference};

/// Gap A: Graph problem — N nodes, N-1 edges (tree input).
///
/// ```text
/// N
/// u_1 v_1
/// u_2 v_2
/// ...
/// u_{N-1} v_{N-1}
/// ```
#[test]
fn e2e_graph_tree_n_minus_1_edges() {
    let mut engine = AstEngine::default();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: n_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: n_id,
        lo: Some(Expression::Lit(5)),
        hi: Some(Expression::Lit(5)),
    });

    let u_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("u"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: u_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: u_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Var(Reference::VariableRef(n_id))),
    });

    let v_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("v"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: v_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: v_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Var(Reference::VariableRef(n_id))),
    });

    let tuple_id = engine.structure.add_node(NodeKind::Tuple {
        elements: vec![u_id, v_id],
    });

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Lit(1)),
        },
        index_var: None,
        body: vec![tuple_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    // Sample generation
    let sample = generate(&engine, 42).unwrap();
    let output = render_sample(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines[0], "5");
    assert_eq!(lines.len(), 5); // N + N-1 edges

    for line in &lines[1..] {
        let parts: Vec<i64> = line
            .split_whitespace()
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(parts.len(), 2);
        assert!((1..=5).contains(&parts[0]));
        assert!((1..=5).contains(&parts[1]));
    }

    // TeX rendering
    let tex_result = render_input_tex(&engine);
    assert!(
        tex_result.tex.contains("N-1"),
        "TeX should show N-1 count: {}",
        tex_result.tex
    );

    // Plain text rendering
    let text = render_input(&engine);
    assert!(text.contains("u_i v_i"), "plain text: {text}");
}

/// Gap H + D: Triangular matrix — row i has N-i elements.
///
/// ```text
/// N
/// C_{0,1} C_{0,2} ... C_{0,N-1}
/// C_{1,2} ... C_{1,N-1}
/// ...
/// C_{N-2,N-1}
/// ```
#[test]
fn e2e_triangular_matrix_via_repeat_loop_var() {
    let mut engine = AstEngine::default();

    let n_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("N"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: n_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: n_id,
        lo: Some(Expression::Lit(4)),
        hi: Some(Expression::Lit(4)),
    });

    // Array C with length = N - i - 1
    let c_id = engine.structure.add_node(NodeKind::Array {
        name: Ident::new("C"),
        length: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::BinOp {
                op: ArithOp::Sub,
                lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
                rhs: Box::new(Expression::Var(Reference::Unresolved(Ident::new("i")))),
            }),
            rhs: Box::new(Expression::Lit(1)),
        },
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: c_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: c_id,
        lo: Some(Expression::Lit(0)),
        hi: Some(Expression::Lit(99)),
    });

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::BinOp {
            op: ArithOp::Sub,
            lhs: Box::new(Expression::Var(Reference::VariableRef(n_id))),
            rhs: Box::new(Expression::Lit(1)),
        },
        index_var: Some(Ident::new("i")),
        body: vec![c_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![n_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let output = render_sample(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();

    // N=4, N-1=3 rows: i=0 → 3 elements, i=1 → 2, i=2 → 1
    assert_eq!(lines[0], "4");
    assert_eq!(lines.len(), 4); // N + 3 rows
    assert_eq!(lines[1].split_whitespace().count(), 3); // N-0-1=3
    assert_eq!(lines[2].split_whitespace().count(), 2); // N-1-1=2
    assert_eq!(lines[3].split_whitespace().count(), 1); // N-2-1=1
}

/// Gap B: Query problem — Q queries with tag-dependent variants.
///
/// ```text
/// Q
/// 1 X_1
/// 2 Y_1 Z_1
/// 1 X_2
/// ...
/// ```
#[test]
fn e2e_query_problem_choice_in_repeat() {
    let mut engine = AstEngine::default();

    let q_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Q"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: q_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: q_id,
        lo: Some(Expression::Lit(20)),
        hi: Some(Expression::Lit(20)),
    });

    let t_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("T"),
    });

    let x_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("X"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: x_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: x_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(1000)),
    });

    let y_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Y"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: y_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: y_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(1000)),
    });

    let z_id = engine.structure.add_node(NodeKind::Scalar {
        name: Ident::new("Z"),
    });
    engine.constraints.add(Constraint::TypeDecl {
        target: z_id,
        expected: ExpectedType::Int,
    });
    engine.constraints.add(Constraint::Range {
        target: z_id,
        lo: Some(Expression::Lit(1)),
        hi: Some(Expression::Lit(1000)),
    });

    let choice_id = engine.structure.add_node(NodeKind::Choice {
        tag: Reference::VariableRef(t_id),
        variants: vec![
            (Literal::IntLit(1), vec![x_id]),
            (Literal::IntLit(2), vec![y_id, z_id]),
        ],
    });

    let repeat_id = engine.structure.add_node(NodeKind::Repeat {
        count: Expression::Var(Reference::VariableRef(q_id)),
        index_var: None,
        body: vec![choice_id],
    });

    let root = engine.structure.root();
    engine
        .structure
        .get_mut(root)
        .unwrap()
        .set_kind(NodeKind::Sequence {
            children: vec![q_id, repeat_id],
        });

    let sample = generate(&engine, 42).unwrap();
    let output = render_sample(&engine, &sample);
    let lines: Vec<&str> = output.trim().lines().collect();

    assert_eq!(lines[0], "20");
    assert_eq!(lines.len(), 21); // Q + 20 query lines

    let mut type1_count = 0;
    let mut type2_count = 0;
    for line in &lines[1..] {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "1" => {
                assert_eq!(parts.len(), 2, "type 1 should have tag + X");
                type1_count += 1;
            }
            "2" => {
                assert_eq!(parts.len(), 3, "type 2 should have tag + Y + Z");
                type2_count += 1;
            }
            other => panic!("unexpected tag: {other}"),
        }
    }
    assert!(type1_count > 0, "should have at least one type-1 query");
    assert!(type2_count > 0, "should have at least one type-2 query");

    // TeX rendering should use cases environment
    let tex_result = render_input_tex(&engine);
    assert!(
        tex_result.tex.contains("\\begin{cases}"),
        "TeX should use cases: {}",
        tex_result.tex
    );

    // Plain text should show If T = k: ...
    let text = render_input(&engine);
    assert!(text.contains("If T = 1:"), "plain text: {text}");
    assert!(text.contains("If T = 2:"), "plain text: {text}");
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test gap_resolution -- --nocapture`

Expected: All 3 PASS.

- [ ] **Step 3: Run full test suite**

Run: `cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-targets`

Expected: All tests pass (204+).

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "test: add end-to-end gap resolution integration tests

- e2e_graph_tree_n_minus_1_edges: Gap A graph pattern
- e2e_triangular_matrix_via_repeat_loop_var: Gap H+D triangular pattern
- e2e_query_problem_choice_in_repeat: Gap B query pattern

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```

---

### Task T-08: Update documentation

Update project documentation to reflect the gap resolution changes.

**Files:**
- Modify: `doc/plan/processing.md`

- [ ] **Step 1: Update processing.md**

Add a new section to `doc/plan/processing.md` documenting the gap resolution phase:

```markdown
## Gap Resolution Phase (Gaps A, B, H, D)

**Status:** Complete

**What was done:**
- `Repeat.count` changed from `Reference` to `Expression` — supports `N-1`, `N*(N-1)/2`, etc.
- `Array.length` changed from `Reference` to `Expression` — supports loop-variable-dependent lengths
- `Repeat.index_var: Option<Ident>` added — loop counter accessible in body expressions
- `resolve_expression_as_int()` in sample generator — full arithmetic expression evaluation
- Choice rendering improved: TeX uses `\begin{cases}...\end{cases}`, plain text uses `If T = k: ...`
- Triangular/jagged matrices expressed as `Repeat(index_var) + Array(Expression)` — no Matrix changes needed

**Gaps addressed:**
- Gap A (P0): Repeat.count accepts Expression (graph problems with N-1 edges)
- Gap B (P1): Choice in Repeat verified and rendering improved (query-type problems)
- Gap H (P2): Loop variable in Repeat (iteration index accessible in body)
- Gap D (P2): Triangular matrix via Repeat+Array with loop variable
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "doc: update processing.md with gap resolution phase

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
```
