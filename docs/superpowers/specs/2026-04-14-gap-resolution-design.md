# Gap Resolution Design Spec — Expression Count, Loop Variable, Choice Rendering

**Date:** 2026-04-14
**Scope:** Gaps A (P0), B (P1), H (P2), D (P2) from AtCoder coverage survey

## 1. Problem Statement

The current AST cannot express several common competitive programming input patterns:

| Gap | Issue | Example |
|-----|-------|---------|
| **A** | `Repeat.count` only accepts `Reference`, not arithmetic expressions | Graph problems: `N-1` edges |
| **B** | `Choice` inside `Repeat` is untested; TeX/plain renderers produce placeholder text | Query-type problems |
| **H** | `Repeat` has no loop variable (iteration index) | Body elements can't reference row index |
| **D** | No triangular/jagged 2D matrix | Cost matrix: row `i` has `N-i` columns |

## 2. Approach

**Approach B (Full):** Extend Repeat with Expression-based count and optional loop variable. Improve Choice renderers. No new NodeKind variants needed — triangular matrices are expressed as `Repeat + Array` with Expression-based length.

## 3. Data Model Changes

### 3.1 NodeKind::Repeat

**Before:**
```rust
Repeat { count: Reference, body: Vec<NodeId> }
```

**After:**
```rust
Repeat {
    count: Expression,
    index_var: Option<Ident>,
    body: Vec<NodeId>,
}
```

- `count` becomes `Expression` — subsumes `Reference` via `Expression::Var(Reference)`
- `index_var` is the iteration counter name (e.g., `Ident("i")`)
- `body` is unchanged

**Migration rule:** All existing `Repeat { count: ref, .. }` become `Repeat { count: Expression::Var(ref), index_var: None, .. }`.

### 3.2 Array.length → Expression

**Before:**
```rust
Array { name: Ident, length: Reference }
```

**After:**
```rust
Array { name: Ident, length: Expression }
```

Required for Gap D (triangular matrix): body Array inside a Repeat needs `length: BinOp(Sub, Var(N), Var(Unresolved("i")))`. Same migration rule as Repeat.count — wrap `Reference` in `Expression::Var()`.

### 3.3 No changes to other NodeKinds

- `Choice` keeps its current structure: `{ tag: Reference, variants: Vec<(Literal, Vec<NodeId>)> }`
- `Matrix` remains rectangular only — triangular patterns use Repeat+Array

## 4. Expression Resolution in Sample Generator

### 4.1 New method: `resolve_expression_as_int`

```rust
fn resolve_expression_as_int(&self, expr: &Expression) -> Result<i64, GenerationError> {
    match expr {
        Expression::Lit(v) => Ok(*v),
        Expression::Var(reference) => {
            // Check loop_vars first for Unresolved references
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
            // Perform checked arithmetic, return InvalidExpression on overflow
            ...
        }
        Expression::Pow { base, exp } => { ... }
        Expression::FnCall { name, args } => {
            // Support min/max; others return error
            ...
        }
    }
}
```

### 4.2 Loop variable scope

Add to `GenerationContext`:
```rust
loop_vars: HashMap<Ident, i64>,
```

In `generate_repeat()`:
```rust
fn generate_repeat(&mut self, node_id, count_expr, index_var, body) {
    let count = self.resolve_expression_as_int(count_expr)?;
    // ... bounds check ...

    for i in 0..count_usize {
        // Set loop variable if present
        if let Some(var_name) = index_var {
            self.loop_vars.insert(var_name.clone(), i as i64);
        }

        // Generate body children
        for &child_id in body { ... }

        // Snapshot + cleanup (existing logic)
        ...
    }

    // Remove loop variable after loop completes
    if let Some(var_name) = index_var {
        self.loop_vars.remove(var_name);
    }
}
```

### 4.3 Dependency graph: Expression reference extraction

New helper function:
```rust
fn extract_var_refs(expr: &Expression) -> Vec<NodeId> {
    match expr {
        Expression::Lit(_) => vec![],
        Expression::Var(Reference::VariableRef(id)) => vec![*id],
        Expression::Var(_) => vec![],  // Unresolved/IndexedRef
        Expression::BinOp { lhs, rhs, .. } => {
            let mut refs = extract_var_refs(lhs);
            refs.extend(extract_var_refs(rhs));
            refs
        }
        Expression::Pow { base, exp } => { ... }
        Expression::FnCall { args, .. } => { ... }
    }
}
```

In `dependency.rs`, the Repeat arm changes from:
```rust
if let Reference::VariableRef(ref_id) = count {
    deps.entry(id).or_default().push(*ref_id);
}
```
to:
```rust
for ref_id in extract_var_refs(count) {
    deps.entry(id).or_default().push(ref_id);
}
```

## 5. Rendering Changes

### 5.1 Expression to TeX

New function `expression_to_tex(engine, expr, warnings) -> String`:
```rust
fn expression_to_tex(engine: &AstEngine, expr: &Expression, warnings: &mut Vec<TexWarning>) -> String {
    match expr {
        Expression::Lit(v) => v.to_string(),
        Expression::Var(reference) => reference_to_tex(engine, reference, warnings),
        Expression::BinOp { op, lhs, rhs } => {
            let l = expression_to_tex(engine, lhs, warnings);
            let r = expression_to_tex(engine, rhs, warnings);
            let op_str = match op {
                ArithOp::Add => "+",
                ArithOp::Sub => "-",
                ArithOp::Mul => "\\times ",
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
            let args_str: Vec<_> = args.iter()
                .map(|a| expression_to_tex(engine, a, warnings))
                .collect();
            format!("\\mathrm{{{}}}({})", name.as_str(), args_str.join(", "))
        }
    }
}
```

Repeat rendering uses `expression_to_tex()` instead of `reference_to_tex()`.

### 5.2 Expression to plain text

Analogous `expression_to_text(engine, expr) -> String` for `input_format.rs`.

### 5.3 Choice TeX rendering

**Before:** `\texttt{(choice)}`

**After:**
```latex
\begin{cases}
X_i \ Y_i \ Z_i & (\text{if } T_i = 1) \\
X_i \ Y_i & (\text{if } T_i = 2) \\
X_i & (\text{if } T_i = 3)
\end{cases}
```

Implementation in `input_tex.rs`:
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
        let lit_str = literal_to_tex(literal);
        let separator = if i + 1 < variants.len() { " \\\\" } else { "" };
        lines.push(format!("{body_str} & (\\text{{if }} {tag_str} = {lit_str}){separator}"));
    }
    lines.push("\\end{cases}".to_owned());
}
```

### 5.4 Choice plain text rendering

**Before:** `(choice)`

**After:**
```
If T_i = 1: X_i Y_i Z_i
If T_i = 2: X_i Y_i
If T_i = 3: X_i
```

### 5.5 Sample output for Choice in Repeat

The `output.rs` already correctly handles Choice output (tag value + chosen variant children). For Choice inside Repeat, the existing `repeat_instances` snapshot mechanism captures all values including Choice children. Verify with tests.

## 6. Files to Modify

| File | Change |
|------|--------|
| `structure/node_kind.rs` | Repeat: `count: Expression`, `index_var: Option<Ident>`; Array: `length: Expression` |
| `sample/dependency.rs` | `extract_var_refs()`, Repeat + Array arms use Expression extraction |
| `sample/generator.rs` | `loop_vars` field, `resolve_expression_as_int()`, `generate_repeat()` + `generate_array()` update |
| `sample/output.rs` | Repeat arm pattern match update (add `index_var`) |
| `render/input_format.rs` | `expression_to_text()`, Choice rendering, Repeat count |
| `render_tex/input_tex.rs` | `expression_to_tex()`, Choice `cases` environment, Repeat count |
| `operation/node_ops.rs` | Repeat arm pattern match update (3 places) |
| `projection/projection_impl.rs` | Repeat arm pattern match update (3 places) |
| `operation/multi_test_case.rs` | Wrap count in `Expression::Var()`, add `index_var: None` |
| All test files | Migrate `Repeat { count: ref, body }` → `Repeat { count: Expression::Var(ref), index_var: None, body }`; `Array { name, length: ref }` → `Array { name, length: Expression::Var(ref) }` |

## 7. Design Notes

- `index_var` is **0-indexed** (matches Rust `0..count` iteration). Problem modelers use `i+1` in expressions for 1-indexed problems.
- `resolve_expression_as_int` resolves `Unresolved` idents against `loop_vars` first, falling through to `resolve_reference_as_int` for node-backed variables.

## 7. Testing Strategy

### 7.1 Unit tests (sample generation)
- **Expression count:** Repeat with `N-1` count, verify N-1 iterations generated
- **Loop variable basic:** Repeat with `index_var: Some("i")`, body Array with `length: Var(Unresolved("i"))` — verify varying row lengths
- **Choice in Repeat:** Repeat containing Choice, verify each iteration picks a variant independently
- **Nested loop vars:** Repeat inside Repeat with different index vars

### 7.2 Integration tests (e2e)
- **Graph problem pattern:** N scalar + Repeat(N-1) { Tuple(u, v) } — typical tree input
- **Triangular matrix pattern:** N scalar + Repeat(N, index_var="i") { Array(length=N-i-1) }
- **Query problem pattern:** Q scalar + Repeat(Q) { Choice(tag=T, variants=...) }

### 7.3 Rendering tests
- Expression count renders as `N-1` in TeX and plain text
- Choice renders as `\begin{cases}...\end{cases}` in TeX
- Choice renders as `If T = k: ...` in plain text

## 8. Out of Scope

- `Array.length: Reference` → also changed to Expression (needed for triangular matrix pattern)
- `Matrix` shape variants (triangular expressed via Repeat+Array instead)
- `Expression.Lit` BigInt (Gap G, P3)
- String pattern constraints (Gap E, P4)
- String length/charset constraints (Gap F, P4)
- Tuple inline arrays (Gap C, P4)
