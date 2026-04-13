# Sample Generation Phase A — Design Spec

**Date:** 2026-04-13
**Scope:** Enhance existing `sample/` module — variable reference resolution, Repeat expansion, Choice branching, Result-based error handling
**Target:** `crates/cp-ast-core/src/sample/`

---

## 1. Problem Statement

The current `sample/` module generates random test cases from AST but has critical limitations:

1. **Expression evaluation is constant-only** — `find_and_resolve_range` uses `evaluate_constant()`, so `1 ≤ A_i ≤ N` (where N is already generated) falls back to default `[1, 100]`.
2. **Repeat body is not expanded** — The body is emitted once regardless of the repeat count.
3. **Choice always picks first variant** — No actual random branching.
4. **`GuaranteeLevel` allows silent constraint violations** — Values may violate constraints with only a warning.

## 2. Design Overview

```
┌─────────────────────────────────────────────────┐
│                 generate()                       │
│  (public API, returns Result<..., GenError>)     │
├─────────────────────────────────────────────────┤
│  DependencyGraph::build()  →  topological_sort() │
├─────────────────────────────────────────────────┤
│  GenerationContext                               │
│  ├── rng: StdRng                                 │
│  ├── values: HashMap<NodeId, SampleValue>        │
│  ├── repeat_instances: HashMap<NodeId, Vec<...>> │
│  └── config: GenerationConfig                    │
├─────────────────────────────────────────────────┤
│  Node generators (methods on GenerationContext)   │
│  ├── generate_scalar()                           │
│  ├── generate_array()                            │
│  ├── generate_matrix()                           │
│  ├── generate_repeat()     ← NEW                 │
│  └── generate_choice()     ← NEW                 │
├─────────────────────────────────────────────────┤
│  Expression resolver (method on GenerationContext)│
│  └── evaluate() — resolves Var refs from values   │
├─────────────────────────────────────────────────┤
│  sample_to_text() — updated for Repeat/Choice     │
└─────────────────────────────────────────────────┘
```

### Approach: Option A + C (Incremental + GenerationContext)

- Introduce `GenerationContext` struct to centralize generation state.
- Migrate existing helper functions to `impl GenerationContext` methods.
- Add new capabilities incrementally (Expression resolution → Repeat → Choice).
- Replace `GuaranteeLevel` with `Result<T, GenerationError>` + internal retry.

## 3. GenerationContext

```rust
pub struct GenerationConfig {
    pub max_retries: u32,        // Default: 100
    pub max_repeat_count: usize, // Default: 500_000
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_retries: 100,
            max_repeat_count: 500_000,
        }
    }
}

struct GenerationContext<'a> {
    engine: &'a AstEngine,
    rng: StdRng,
    values: HashMap<NodeId, SampleValue>,
    repeat_instances: HashMap<NodeId, Vec<HashMap<NodeId, SampleValue>>>,
    config: GenerationConfig,
}
```

**Key design decisions:**

- `GenerationContext` is **not public** — it's an internal implementation detail. The public API remains `generate()`.
- `repeat_instances` stores per-Repeat-node expansion results, keyed by the Repeat node's `NodeId`. Each entry is a `Vec` of `HashMap<NodeId, SampleValue>` (one map per iteration).
- `engine` is borrowed to avoid cloning.
- `config` allows tuning retry limits (extensible for future phases).

## 4. Expression Resolution

### 4.1 New method: `GenerationContext::evaluate()`

```rust
impl GenerationContext<'_> {
    fn evaluate(&self, expr: &Expression) -> Result<i64, GenerationError> {
        match expr {
            Expression::Lit(v) => Ok(*v),
            Expression::Var(reference) => self.resolve_var_reference(reference),
            Expression::BinOp { op, lhs, rhs } => {
                let l = self.evaluate(lhs)?;
                let r = self.evaluate(rhs)?;
                apply_arith_op(*op, l, r)
            }
            Expression::Pow { base, exp } => {
                let b = self.evaluate(base)?;
                let e = self.evaluate(exp)?;
                let e_u32 = u32::try_from(e)
                    .map_err(|_| GenerationError::InvalidExpression("negative exponent".into()))?;
                b.checked_pow(e_u32)
                    .ok_or(GenerationError::InvalidExpression("overflow in pow".into()))
            }
            Expression::FnCall { name, args } => {
                self.evaluate_fn_call(name, args)
            }
        }
    }

    fn resolve_var_reference(&self, reference: &Reference) -> Result<i64, GenerationError> {
        match reference {
            Reference::VariableRef(id) => {
                match self.values.get(id) {
                    Some(SampleValue::Int(v)) => Ok(*v),
                    Some(other) => Err(GenerationError::TypeMismatch {
                        node_id: *id,
                        expected: "Int",
                        got: format!("{other:?}"),
                    }),
                    None => Err(GenerationError::UnresolvedReference(*id)),
                }
            }
            Reference::IndexedRef { .. } => {
                // IndexedRef depends on loop variables or quantified indices,
                // which have no concrete value at generation time.
                // Phase A explicitly does NOT support indexed reference resolution.
                Err(GenerationError::InvalidExpression(
                    "indexed reference in generation context (unsupported in Phase A)".into()
                ))
            }
            Reference::Unresolved(name) => {
                Err(GenerationError::InvalidExpression(
                    format!("unresolved name: {}", name.as_str())
                ))
            }
        }
    }

    fn evaluate_fn_call(
        &self,
        name: &Ident,
        args: &[Expression],
    ) -> Result<i64, GenerationError> {
        let evaluated: Vec<i64> = args.iter().map(|a| self.evaluate(a)).collect::<Result<_, _>>()?;
        match name.as_str() {
            "min" => evaluated.iter().copied().min()
                .ok_or(GenerationError::InvalidExpression("min() with no args".into())),
            "max" => evaluated.iter().copied().max()
                .ok_or(GenerationError::InvalidExpression("max() with no args".into())),
            "abs" => {
                if evaluated.len() != 1 {
                    return Err(GenerationError::InvalidExpression("abs() requires 1 arg".into()));
                }
                evaluated[0].checked_abs()
                    .ok_or(GenerationError::InvalidExpression("abs() overflow (i64::MIN)".into()))
            }
            _ => Err(GenerationError::InvalidExpression(
                format!("unknown function: {}", name.as_str())
            )),
        }
    }
}
```

### 4.2 Range resolution migration

Replace `find_and_resolve_range()` free function with:

```rust
impl GenerationContext<'_> {
    fn resolve_range(&self, constraints: &[&Constraint]) -> Result<(i64, i64), GenerationError> {
        let range = constraints.iter().find_map(|c| {
            if let Constraint::Range { lower, upper, .. } = c {
                Some((lower.clone(), upper.clone()))
            } else {
                None
            }
        });

        if let Some((lower, upper)) = range {
            let lo = self.evaluate(&lower)?;
            let hi = self.evaluate(&upper)?;
            if lo > hi {
                // Do NOT silently swap — this indicates an unsatisfiable constraint
                // or a construction error. Propagate as error.
                Err(GenerationError::RangeEmpty { min: lo, max: hi })
            } else {
                Ok((lo, hi))
            }
        } else {
            Ok((1, 100)) // default range
        }
    }
}
```

Similarly, `resolve_string_length()` and `resolve_reference_as_length()` become methods on `GenerationContext`.

## 5. Repeat Node Expansion

### 5.1 Generation

When the topological sort reaches a Repeat node, `GenerationContext` must:

1. Evaluate `count` (a `Reference`) to get the iteration count N.
2. For each iteration `i` in `0..N`:
   a. Generate all body child nodes.
   b. Store body values in a per-iteration map.
3. Store the collected iterations in `repeat_instances`.

```rust
fn generate_repeat(&mut self, node_id: NodeId, count_ref: &Reference, body: &[NodeId])
    -> Result<(), GenerationError>
{
    let count = self.resolve_reference_as_int(count_ref)?;
    let count_usize = usize::try_from(count)
        .map_err(|_| GenerationError::InvalidExpression("negative repeat count".into()))?;

    // Guard against absurdly large repeat counts
    if count_usize > self.config.max_repeat_count {
        return Err(GenerationError::InvalidStructure(format!(
            "repeat count {count_usize} exceeds limit {}",
            self.config.max_repeat_count
        )));
    }

    let mut instances = Vec::with_capacity(count_usize);

    for _i in 0..count_usize {
        // Generate body children into self.values so they can reference
        // each other within the same iteration (e.g., Y depends on X).
        for &child_id in body {
            if let Some(node) = self.engine.structure.get(child_id) {
                let kind = node.kind().clone();
                self.generate_node_inner(child_id, &kind)?;
            }
        }

        // Snapshot: copy body child values into iteration map
        let mut iteration_values = HashMap::new();
        for &child_id in body {
            if let Some(val) = self.values.get(&child_id) {
                iteration_values.insert(child_id, val.clone());
            }
        }
        instances.push(iteration_values);

        // Remove body child values from self.values to prepare for next iteration.
        // This ensures the next iteration generates fresh values, not reusing old ones.
        for &child_id in body {
            self.values.remove(&child_id);
        }
    }

    self.repeat_instances.insert(node_id, instances);
    Ok(())
}
```

**Key design: local scope pattern for Repeat iterations.**

Within a single iteration, body children are generated into `self.values` so that
later children can reference earlier children (e.g., `1 ≤ Y ≤ X` where both X and
Y are in the same Repeat body). After all children are generated, values are
snapshot-copied into the iteration map. Then child values are removed from
`self.values` to prepare for the next iteration.

This means `evaluate()` naturally resolves intra-iteration references through the
normal `self.values` lookup path — no separate local scope map is needed.
```

### 5.2 Dependency graph adjustment

The current `DependencyGraph::build()` already handles Repeat: body children depend on the Repeat node, and the Repeat node depends on its `count` reference. No structural changes needed.

However, body children of a Repeat **must not** be generated during the main topological walk — they are generated inside `generate_repeat()`. The main loop must **skip** nodes that are body children of any Repeat node.

```rust
// In the main generate() loop:
let skip_set: HashSet<NodeId> = build_skip_set(engine);
for node_id in &order {
    if skip_set.contains(node_id) {
        continue; // Generated inside generate_repeat() or generate_choice()
    }
    // ... normal generation
}
```

### 5.3 Constraint handling within Repeat body

Body children inherit their own constraints (Range, TypeDecl, etc.) from `ConstraintSet::for_node()`. Each iteration generates fresh values independently. No per-iteration constraint state is needed for Phase A.

## 6. Choice Node Branching

```rust
fn generate_choice(
    &mut self,
    node_id: NodeId,
    tag_ref: &Reference,
    variants: &[(Literal, Vec<NodeId>)],
) -> Result<(), GenerationError> {
    if variants.is_empty() {
        return Err(GenerationError::InvalidStructure(
            "Choice node has no variants".into()
        ));
    }

    // Select a random variant
    let idx = self.rng.gen_range(0..variants.len());
    let (tag_value, children) = &variants[idx];

    // Store the tag value
    // Resolve tag reference to find which node to write the tag to
    if let Reference::VariableRef(tag_id) = tag_ref {
        let tag_sample = match tag_value {
            Literal::IntLit(v) => SampleValue::Int(*v),
            Literal::StrLit(s) => SampleValue::Str(s.clone()),
        };
        self.values.insert(*tag_id, tag_sample);
    }

    // Generate the chosen variant's children
    for &child_id in children {
        if let Some(node) = self.engine.structure.get(child_id) {
            let kind = node.kind().clone();
            self.generate_node_inner(child_id, &kind)?;
        }
    }

    Ok(())
}
```

Choice node children (non-selected variants) are **not** generated. Like Repeat body children, Choice variant children must be skipped in the main topological walk.

**Tag ownership rule:** The Choice node **owns** its tag. The tag node is included in the skip set and is NOT generated as a normal Scalar during the main topological walk. Only `generate_choice()` writes the tag value.

```rust
let skip_set: HashSet<NodeId> = {
    let mut s = HashSet::new();
    // Repeat body children
    s.extend(collect_repeat_body_children(engine));
    // Choice variant children
    s.extend(collect_choice_variant_children(engine));
    // Choice tag nodes (owned by their Choice node)
    s.extend(collect_choice_tag_nodes(engine));
    s
};
```

## 7. Error Handling

### 7.1 GenerationError

```rust
#[derive(Debug, Clone)]
pub enum GenerationError {
    /// Dependency cycle detected.
    CycleDetected(CycleError),
    /// Variable reference could not be resolved (not yet generated).
    UnresolvedReference(NodeId),
    /// Type mismatch when resolving a reference.
    TypeMismatch {
        node_id: NodeId,
        expected: &'static str,
        got: String,
    },
    /// Range is empty after resolution (min > max with no valid swap).
    RangeEmpty { min: i64, max: i64 },
    /// Retry limit exhausted for a node.
    RetryExhausted { node_id: NodeId, attempts: u32 },
    /// Expression evaluation failed.
    InvalidExpression(String),
    /// Structural issue (e.g., Choice with no variants).
    InvalidStructure(String),
}
```

- Implements `std::fmt::Display` and `std::error::Error`.
- `CycleDetected` wraps the existing `CycleError` (preserving backward compatibility of the dependency module).

### 7.2 Internal retry

Used for stochastic operations (distinct array generation, graph edge generation):

```rust
fn with_retry<F, T>(&mut self, node_id: NodeId, mut f: F) -> Result<T, GenerationError>
where
    F: FnMut(&mut Self) -> Result<T, GenerationError>,
{
    let max = self.config.max_retries;
    for attempt in 0..max {
        match f(self) {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt + 1 >= max {
                    return Err(GenerationError::RetryExhausted {
                        node_id,
                        attempts: max,
                    });
                }
                // On retryable errors, continue; on fatal errors, propagate immediately
                if !is_retryable(&e) {
                    return Err(e);
                }
            }
        }
    }
    unreachable!()
}
```

Retryable errors: distinct generation failure, graph edge generation failure (stochastic — new random values may succeed).

Non-retryable errors: all `GenerationError` variants (structural / deterministic). See §7.3 for the full classification table.

### 7.3 Retryable error classification

| Error variant | Retryable? | Reason |
|---|---|---|
| `CycleDetected` | No | Structural — cannot change with retry |
| `UnresolvedReference` | No | Missing dependency — deterministic |
| `TypeMismatch` | No | Wrong type — deterministic |
| `RangeEmpty` | No | Constraint is unsatisfiable — retry won't help |
| `RetryExhausted` | No | Already exhausted retries |
| `InvalidExpression` | No | Expression evaluation failure — deterministic |
| `InvalidStructure` | No | AST structural issue — deterministic |

**Note:** In Phase A, the `with_retry` wrapper is used exclusively for **distinct array generation** and **graph edge generation**, where the stochastic nature of rejection sampling can cause transient failures. The retry mechanism re-runs the closure with the same `GenerationContext` (RNG state has advanced), giving a fresh random sample.

If the closure returns any error variant, it is checked:
- If the error is from the distinct/graph generation logic itself (not one of the above enum variants, but rather a "not enough values generated" condition), retry continues.
- If the error is any of the above `GenerationError` variants, it is propagated immediately without retry.

## 8. Public API Changes

### 8.1 generate()

```rust
// Before:
pub fn generate(engine: &AstEngine, seed: u64) -> GeneratedSample;

// After:
pub fn generate(engine: &AstEngine, seed: u64) -> Result<GeneratedSample, GenerationError>;

// With config:
pub fn generate_with_config(
    engine: &AstEngine,
    seed: u64,
    config: GenerationConfig,
) -> Result<GeneratedSample, GenerationError>;
```

### 8.2 GeneratedSample

```rust
// Before:
pub struct GeneratedSample {
    pub values: HashMap<NodeId, SampleValue>,
    pub warnings: Vec<String>,
    pub guarantee_level: GuaranteeLevel,
}

// After:
pub struct GeneratedSample {
    pub values: HashMap<NodeId, SampleValue>,
    pub repeat_instances: HashMap<NodeId, Vec<HashMap<NodeId, SampleValue>>>,
}
```

- `warnings` removed — errors are now surfaced via `Result`.
- `guarantee_level` removed — all returned samples satisfy constraints or `Err` is returned.
- `repeat_instances` added — stores per-Repeat-node iteration data.

### 8.3 SampleValue (unchanged)

No new variants needed. Repeat data is stored in `repeat_instances`, not in `SampleValue`. Individual values within each repeat iteration are standard `SampleValue` types.

### 8.4 GuaranteeLevel

**Removed entirely.** All references to `GuaranteeLevel` and `demote_guarantee()` are deleted.

### 8.5 mod.rs re-exports

```rust
pub use generator::{
    generate, generate_with_config,
    GeneratedSample, GenerationConfig, GenerationError, SampleValue,
};
```

`GuaranteeLevel` is no longer exported.

## 9. sample_to_text() Updates

### 9.1 Repeat rendering

```rust
NodeKind::Repeat { count: _, body } => {
    let body = body.clone();
    if let Some(instances) = sample.repeat_instances.get(&node_id) {
        for iteration_values in instances {
            // Temporarily merge iteration values into sample for child rendering
            for &child_id in &body {
                emit_node_with_values(engine, child_id, iteration_values, output);
            }
        }
    }
}
```

### 9.2 Choice rendering

```rust
NodeKind::Choice { tag, variants } => {
    let tag = tag.clone();
    let variants = variants.clone();
    // Find which variant was selected by checking tag value
    if let Reference::VariableRef(tag_id) = &tag {
        if let Some(tag_val) = sample.values.get(tag_id) {
            for (lit, children) in &variants {
                if literal_matches_value(lit, tag_val) {
                    for &child_id in children {
                        emit_node(engine, child_id, sample, output);
                    }
                    return;
                }
            }
        }
    }
    // Fallback: emit first variant
    if let Some((_, children)) = variants.first() {
        for &child_id in children {
            emit_node(engine, child_id, sample, output);
        }
    }
}
```

## 10. Migration Path

### Existing tests

All existing tests call `generate()` which currently returns `GeneratedSample` directly. After the change, they must call `generate().unwrap()` or `generate().expect(...)`. Since these are test-only usages, `unwrap()` is acceptable.

### Existing callers (e2e, typical_problem tests)

Same pattern — add `.unwrap()` or `.expect()` after `generate()` calls. These tests construct valid ASTs, so `generate()` should succeed.

### sample_to_text()

Signature changes from `sample_to_text(engine, &sample)` to the same — but `GeneratedSample` now has `repeat_instances` field. The output module must be updated to use it.

## 11. Files to Modify

| File | Changes |
|---|---|
| `sample/generator.rs` | Add `GenerationContext`, `GenerationError`, `GenerationConfig`; migrate helpers to methods; implement Repeat/Choice generation; remove `GuaranteeLevel` |
| `sample/output.rs` | Update `emit_node` for Repeat (use `repeat_instances`) and Choice (match selected variant) |
| `sample/mod.rs` | Update re-exports |
| `tests/sample_basic.rs` | Update calls to `generate()` (add `.unwrap()`); add new tests for variable resolution, Repeat, Choice, error cases |
| `tests/e2e_abc284c.rs` | Update `generate()` call |
| `tests/typical_problem.rs` | Update `generate()` call |

## 12. New Test Cases

### Expression resolution
- `1 ≤ A_i ≤ N` with N previously generated → A values in correct range
- `Expression::BinOp(Mul, Var(N), Lit(2))` → evaluates to `2*N`
- `Expression::Pow` with variable base → correct result
- `Expression::FnCall("min", [Var(N), Lit(100)])` → min(N, 100)
- Unresolved reference → `GenerationError::UnresolvedReference`

### Repeat expansion
- Repeat(count=N, body=[Tuple(u_i, v_i)]) with N=3 → 3 iterations in `repeat_instances`
- `sample_to_text` outputs N lines of "u v" pairs
- Repeat with count=0 → empty instances, no output
- Body elements respect their own Range constraints per iteration

### Choice branching
- Choice(tag=T, variants=[(1, [X]), (2, [Y, Z])]) → tag value matches selected variant
- All variants reachable (statistical test with many seeds)
- Variant children are generated, non-selected children are not
- `sample_to_text` renders only the selected variant

### Error handling
- Cycle detection → `GenerationError::CycleDetected`
- Empty range (min > max, unswappable) → `GenerationError::RangeEmpty`
- Distinct array with impossible constraints (range too small) → `GenerationError::RetryExhausted`

### Backward compatibility
- All existing `sample_basic` tests pass (with `.unwrap()` added)
- Deterministic generation with same seed still produces same output

### Additional tests (from feedback)
- Same seed + same AST + same config → identical output (strengthened determinism test)
- Different seeds → Choice variant selection varies (statistical coverage)
- Repeat count exceeding `max_repeat_count` → `GenerationError::InvalidStructure`
- Body child inter-reference within Repeat iteration (e.g., `1 ≤ Y ≤ X` where both are body children)

---

## 13. Non-Goals and Limitations (Phase B/C)

These are explicitly **out of scope** for Phase A:

- SumBound constraint enforcement
- Connected graph generation
- Relation constraint (A < B ordering)
- Post-generation constraint verification
- Backtracking / constraint propagation
- Edge case generation (min/max scenarios)
- Multi-test-case support

### Phase A Limitations (documented for future phases)

1. **`IndexedRef` is unsupported in expression evaluation.** Indexed references (e.g., `A[i]`) depend on loop variables or quantified indices, which have no concrete value at generation time. Phase A returns `InvalidExpression` for these.

2. **Nested Repeat / nested Choice is not fully general.** The `repeat_instances: HashMap<NodeId, Vec<HashMap<NodeId, SampleValue>>>` structure is flat — it does not support Repeat-inside-Repeat or Choice-inside-Repeat with their own iteration state. Phase A handles top-level and simple nesting (Tuple inside Repeat, Scalar inside Repeat). Deeply nested structured output is a Phase B redesign candidate.

3. **`sample_to_text()` value source abstraction.** Currently, `emit_node()` switches between `sample.values` and `sample.repeat_instances` with special-case logic. A future refactor could introduce a unified value view abstraction that `emit_node()` queries, making it extensible for new node types without adding more special cases.
