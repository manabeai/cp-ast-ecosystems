use std::collections::{HashMap, HashSet};
use std::fmt;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::dependency::{CycleError, DependencyGraph};
use crate::constraint::{
    ArithOp, CharSetSpec, Constraint, ExpectedType, Expression, PropertyTag, SortOrder,
};
use crate::operation::AstEngine;
use crate::structure::{Ident, Literal, NodeId, NodeKind, Reference};

/// Error during sample generation.
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
    /// Range is empty after resolution (min > max).
    RangeEmpty { min: i64, max: i64 },
    /// Retry limit exhausted for a node.
    RetryExhausted { node_id: NodeId, attempts: u32 },
    /// Expression evaluation failed.
    InvalidExpression(String),
    /// Structural issue (e.g., Choice with no variants).
    InvalidStructure(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CycleDetected(e) => write!(f, "cycle detected: {e}"),
            Self::UnresolvedReference(id) => write!(f, "unresolved reference: {id:?}"),
            Self::TypeMismatch {
                node_id,
                expected,
                got,
            } => {
                write!(
                    f,
                    "type mismatch at {node_id:?}: expected {expected}, got {got}"
                )
            }
            Self::RangeEmpty { min, max } => write!(f, "empty range: [{min}, {max}]"),
            Self::RetryExhausted { node_id, attempts } => {
                write!(
                    f,
                    "retry exhausted at {node_id:?} after {attempts} attempts"
                )
            }
            Self::InvalidExpression(msg) => write!(f, "invalid expression: {msg}"),
            Self::InvalidStructure(msg) => write!(f, "invalid structure: {msg}"),
        }
    }
}

impl std::error::Error for GenerationError {}

/// Configuration for sample generation.
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    /// Maximum retries for stochastic operations (distinct arrays, graph edges).
    pub max_retries: u32,
    /// Maximum repeat count before rejecting as too large.
    pub max_repeat_count: usize,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_retries: 100,
            max_repeat_count: 500_000,
        }
    }
}

/// A single generated value.
#[derive(Debug, Clone, PartialEq)]
pub enum SampleValue {
    /// Integer value.
    Int(i64),
    /// String value.
    Str(String),
    /// One-dimensional array of values.
    Array(Vec<SampleValue>),
    /// Two-dimensional grid of values.
    Grid(Vec<Vec<SampleValue>>),
}

/// A complete generated sample for a problem.
#[derive(Debug, Clone)]
pub struct GeneratedSample {
    /// Generated values keyed by node ID.
    pub values: HashMap<NodeId, SampleValue>,
    /// Per-Repeat-node iteration data.
    pub repeat_instances: HashMap<NodeId, Vec<HashMap<NodeId, SampleValue>>>,
}

struct GenerationContext<'a> {
    engine: &'a AstEngine,
    rng: StdRng,
    values: HashMap<NodeId, SampleValue>,
    repeat_instances: HashMap<NodeId, Vec<HashMap<NodeId, SampleValue>>>,
    config: GenerationConfig,
}

impl<'a> GenerationContext<'a> {
    fn new(engine: &'a AstEngine, seed: u64, config: GenerationConfig) -> Self {
        Self {
            engine,
            rng: StdRng::seed_from_u64(seed),
            values: HashMap::new(),
            repeat_instances: HashMap::new(),
            config,
        }
    }

    fn into_sample(self) -> GeneratedSample {
        GeneratedSample {
            values: self.values,
            repeat_instances: self.repeat_instances,
        }
    }

    fn evaluate(&self, expr: &Expression) -> Result<i64, GenerationError> {
        match expr {
            Expression::Lit(v) => Ok(*v),
            Expression::Var(reference) => self.resolve_var_reference(reference),
            Expression::BinOp { op, lhs, rhs } => {
                let l = self.evaluate(lhs)?;
                let r = self.evaluate(rhs)?;
                Self::apply_arith_op(*op, l, r)
            }
            Expression::Pow { base, exp } => {
                let b = self.evaluate(base)?;
                let e = self.evaluate(exp)?;
                let e_u32 = u32::try_from(e)
                    .map_err(|_| GenerationError::InvalidExpression("negative exponent".into()))?;
                b.checked_pow(e_u32)
                    .ok_or_else(|| GenerationError::InvalidExpression("overflow in pow".into()))
            }
            Expression::FnCall { name, args } => self.evaluate_fn_call(name, args),
        }
    }

    fn resolve_var_reference(&self, reference: &Reference) -> Result<i64, GenerationError> {
        match reference {
            Reference::VariableRef(id) => match self.values.get(id) {
                Some(SampleValue::Int(v)) => Ok(*v),
                Some(other) => Err(GenerationError::TypeMismatch {
                    node_id: *id,
                    expected: "Int",
                    got: format!("{other:?}"),
                }),
                None => Err(GenerationError::UnresolvedReference(*id)),
            },
            Reference::IndexedRef { .. } => Err(GenerationError::InvalidExpression(
                "indexed reference in generation context (unsupported in Phase A)".into(),
            )),
            Reference::Unresolved(name) => Err(GenerationError::InvalidExpression(format!(
                "unresolved name: {}",
                name.as_str()
            ))),
        }
    }

    fn apply_arith_op(op: ArithOp, l: i64, r: i64) -> Result<i64, GenerationError> {
        let result = match op {
            ArithOp::Add => l.checked_add(r),
            ArithOp::Sub => l.checked_sub(r),
            ArithOp::Mul => l.checked_mul(r),
            ArithOp::Div => {
                if r == 0 {
                    return Err(GenerationError::InvalidExpression(
                        "division by zero".into(),
                    ));
                }
                l.checked_div(r)
            }
        };
        result.ok_or_else(|| GenerationError::InvalidExpression("arithmetic overflow".into()))
    }

    fn evaluate_fn_call(&self, name: &Ident, args: &[Expression]) -> Result<i64, GenerationError> {
        let evaluated: Vec<i64> = args
            .iter()
            .map(|a| self.evaluate(a))
            .collect::<Result<_, _>>()?;
        match name.as_str() {
            "min" => evaluated
                .iter()
                .copied()
                .min()
                .ok_or_else(|| GenerationError::InvalidExpression("min() with no args".into())),
            "max" => evaluated
                .iter()
                .copied()
                .max()
                .ok_or_else(|| GenerationError::InvalidExpression("max() with no args".into())),
            "abs" => {
                if evaluated.len() != 1 {
                    return Err(GenerationError::InvalidExpression(
                        "abs() requires 1 arg".into(),
                    ));
                }
                evaluated[0].checked_abs().ok_or_else(|| {
                    GenerationError::InvalidExpression("abs() overflow (i64::MIN)".into())
                })
            }
            _ => Err(GenerationError::InvalidExpression(format!(
                "unknown function: {}",
                name.as_str()
            ))),
        }
    }

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
                Err(GenerationError::RangeEmpty { min: lo, max: hi })
            } else {
                Ok((lo, hi))
            }
        } else {
            Ok((1, 100))
        }
    }

    fn resolve_reference_as_int(&self, reference: &Reference) -> Result<i64, GenerationError> {
        match reference {
            Reference::VariableRef(id) => match self.values.get(id) {
                Some(SampleValue::Int(v)) => Ok(*v),
                Some(other) => Err(GenerationError::TypeMismatch {
                    node_id: *id,
                    expected: "Int",
                    got: format!("{other:?}"),
                }),
                None => Err(GenerationError::UnresolvedReference(*id)),
            },
            Reference::IndexedRef { .. } => Err(GenerationError::InvalidExpression(
                "indexed reference as length (unsupported in Phase A)".into(),
            )),
            Reference::Unresolved(name) => Err(GenerationError::InvalidExpression(format!(
                "unresolved name: {}",
                name.as_str()
            ))),
        }
    }

    fn resolve_expression_as_int_shim(&self, expr: &Expression) -> Result<i64, GenerationError> {
        match expr {
            Expression::Var(reference) => self.resolve_reference_as_int(reference),
            Expression::Lit(v) => Ok(*v),
            _ => Err(GenerationError::InvalidExpression(
                "complex expressions not yet supported".into(),
            )),
        }
    }

    fn resolve_string_length(
        &mut self,
        constraints: &[&Constraint],
    ) -> Result<usize, GenerationError> {
        for c in constraints {
            if let Constraint::StringLength { min, max, .. } = c {
                let lo = self.evaluate(min)?;
                let hi = self.evaluate(max)?;
                let lo_usize = usize::try_from(lo.max(1)).unwrap_or(1);
                let hi_usize = usize::try_from(hi.max(lo)).unwrap_or(10);
                return Ok(self.rng.gen_range(lo_usize..=hi_usize));
            }
        }
        Ok(self.rng.gen_range(1..=10))
    }

    fn build_skip_set(&self) -> HashSet<NodeId> {
        let mut skip = HashSet::new();
        for node in self.engine.structure.iter() {
            match node.kind() {
                NodeKind::Repeat { body, .. } => {
                    for &child in body {
                        skip.insert(child);
                    }
                }
                NodeKind::Choice { tag, variants } => {
                    if let Reference::VariableRef(tag_id) = tag {
                        skip.insert(*tag_id);
                    }
                    for (_, children) in variants {
                        for &child in children {
                            skip.insert(child);
                        }
                    }
                }
                _ => {}
            }
        }
        skip
    }

    fn generate_node_inner(
        &mut self,
        node_id: NodeId,
        kind: &NodeKind,
    ) -> Result<(), GenerationError> {
        match kind {
            NodeKind::Scalar { .. } => self.generate_scalar(node_id),
            NodeKind::Array { length, .. } => {
                let length = length.clone();
                self.generate_array(node_id, &length)
            }
            NodeKind::Matrix { rows, cols, .. } => {
                let rows = rows.clone();
                let cols = cols.clone();
                self.generate_matrix(node_id, &rows, &cols)
            }
            NodeKind::Repeat {
                count,
                index_var,
                body,
            } => {
                let count = count.clone();
                let index_var = index_var.as_ref();
                let body = body.clone();
                self.generate_repeat(node_id, &count, index_var, &body)
            }
            NodeKind::Choice { tag, variants } => {
                let tag = tag.clone();
                let variants = variants.clone();
                self.generate_choice(node_id, &tag, &variants)
            }
            NodeKind::Sequence { .. }
            | NodeKind::Section { .. }
            | NodeKind::Tuple { .. }
            | NodeKind::Hole { .. } => Ok(()),
        }
    }

    fn generate_scalar(&mut self, node_id: NodeId) -> Result<(), GenerationError> {
        let constraints = get_node_constraints(self.engine, node_id);

        let expected_type = constraints.iter().find_map(|c| {
            if let Constraint::TypeDecl { expected, .. } = c {
                Some(expected.clone())
            } else {
                None
            }
        });

        match expected_type.as_ref().unwrap_or(&ExpectedType::Int) {
            ExpectedType::Int => {
                let (lo, hi) = self.resolve_range(&constraints)?;
                let value = self.rng.gen_range(lo..=hi);
                self.values.insert(node_id, SampleValue::Int(value));
            }
            ExpectedType::Str => {
                let len = self.resolve_string_length(&constraints)?;
                let charset = resolve_charset(&constraints);
                let s: String = (0..len)
                    .map(|_| random_char_from_spec(&charset, &mut self.rng))
                    .collect();
                self.values.insert(node_id, SampleValue::Str(s));
            }
            ExpectedType::Char => {
                let charset = resolve_charset(&constraints);
                let c = random_char_from_spec(&charset, &mut self.rng);
                self.values.insert(node_id, SampleValue::Str(c.to_string()));
            }
        }
        Ok(())
    }

    fn generate_array(
        &mut self,
        node_id: NodeId,
        length_expr: &Expression,
    ) -> Result<(), GenerationError> {
        let len = self.resolve_expression_as_int_shim(length_expr)?;
        let constraints = get_node_constraints(self.engine, node_id);
        let (lo, hi) = self.resolve_range(&constraints)?;

        let is_distinct = constraints
            .iter()
            .any(|c| matches!(c, Constraint::Distinct { .. }));

        let sort_order = constraints.iter().find_map(|c| {
            if let Constraint::Sorted { order, .. } = c {
                Some(*order)
            } else {
                None
            }
        });

        let property_tag = constraints.iter().find_map(|c| {
            if let Constraint::Property { tag, .. } = c {
                Some(tag.clone())
            } else {
                None
            }
        });

        let len_usize = usize::try_from(len).unwrap_or(0);

        let mut elements = if let Some(tag) = &property_tag {
            generate_property_array(tag, len_usize, lo, hi, &mut self.rng)
        } else if is_distinct {
            self.generate_distinct_array(node_id, len_usize, lo, hi)?
        } else {
            (0..len_usize)
                .map(|_| SampleValue::Int(self.rng.gen_range(lo..=hi)))
                .collect()
        };

        if let Some(order) = sort_order {
            sort_sample_values(&mut elements, order);
        }

        self.values.insert(node_id, SampleValue::Array(elements));
        Ok(())
    }

    fn generate_distinct_array(
        &mut self,
        node_id: NodeId,
        len: usize,
        lo: i64,
        hi: i64,
    ) -> Result<Vec<SampleValue>, GenerationError> {
        let range_size = hi.saturating_sub(lo).saturating_add(1);

        if i64::try_from(len).unwrap_or(i64::MAX) > range_size {
            return Err(GenerationError::RetryExhausted {
                node_id,
                attempts: 0,
            });
        }

        if range_size <= 100_000 {
            // Fisher-Yates
            let mut pool: Vec<i64> = (lo..=hi).collect();
            let pick = len.min(pool.len());
            for i in 0..pick {
                let j = self.rng.gen_range(i..pool.len());
                pool.swap(i, j);
            }
            Ok(pool.into_iter().take(len).map(SampleValue::Int).collect())
        } else {
            // Rejection sampling with retry
            let max_attempts = self.config.max_retries as usize * len;
            let mut seen = HashSet::with_capacity(len);
            let mut result = Vec::with_capacity(len);
            let mut attempts = 0;

            while result.len() < len && attempts < max_attempts {
                let v = self.rng.gen_range(lo..=hi);
                if seen.insert(v) {
                    result.push(SampleValue::Int(v));
                }
                attempts += 1;
            }

            if result.len() < len {
                Err(GenerationError::RetryExhausted {
                    node_id,
                    attempts: self.config.max_retries,
                })
            } else {
                Ok(result)
            }
        }
    }

    fn generate_matrix(
        &mut self,
        node_id: NodeId,
        rows_ref: &Reference,
        cols_ref: &Reference,
    ) -> Result<(), GenerationError> {
        let rows = self.resolve_reference_as_int(rows_ref)?;
        let cols = self.resolve_reference_as_int(cols_ref)?;
        let constraints = get_node_constraints(self.engine, node_id);
        let (lo, hi) = self.resolve_range(&constraints)?;

        let rows_usize = usize::try_from(rows).unwrap_or(0);
        let cols_usize = usize::try_from(cols).unwrap_or(0);

        let grid: Vec<Vec<SampleValue>> = (0..rows_usize)
            .map(|_| {
                (0..cols_usize)
                    .map(|_| SampleValue::Int(self.rng.gen_range(lo..=hi)))
                    .collect()
            })
            .collect();

        self.values.insert(node_id, SampleValue::Grid(grid));
        Ok(())
    }

    fn generate_repeat(
        &mut self,
        node_id: NodeId,
        count_expr: &Expression,
        _index_var: Option<&Ident>,
        body: &[NodeId],
    ) -> Result<(), GenerationError> {
        let count = self.resolve_expression_as_int_shim(count_expr)?;
        let count_usize = usize::try_from(count)
            .map_err(|_| GenerationError::InvalidExpression("negative repeat count".into()))?;

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

            // Remove body child values to prepare for next iteration
            for &child_id in body {
                self.values.remove(&child_id);
            }
        }

        self.repeat_instances.insert(node_id, instances);
        Ok(())
    }

    fn generate_choice(
        &mut self,
        _node_id: NodeId,
        tag_ref: &Reference,
        variants: &[(Literal, Vec<NodeId>)],
    ) -> Result<(), GenerationError> {
        if variants.is_empty() {
            return Err(GenerationError::InvalidStructure(
                "Choice node has no variants".into(),
            ));
        }

        let idx = self.rng.gen_range(0..variants.len());
        let (tag_value, children) = &variants[idx];

        // Store the tag value (Choice owns its tag node)
        if let Reference::VariableRef(tag_id) = tag_ref {
            let tag_sample = match tag_value {
                Literal::IntLit(v) => SampleValue::Int(*v),
                Literal::StrLit(s) => SampleValue::Str(s.clone()),
            };
            self.values.insert(*tag_id, tag_sample);
        }

        // Generate the chosen variant's children
        let children = children.clone();
        for child_id in &children {
            if let Some(node) = self.engine.structure.get(*child_id) {
                let kind = node.kind().clone();
                self.generate_node_inner(*child_id, &kind)?;
            }
        }

        Ok(())
    }
}

/// Generate a sample from an `AstEngine`, using a deterministic seed.
///
/// # Errors
/// Returns `GenerationError` if constraints cannot be satisfied.
pub fn generate(engine: &AstEngine, seed: u64) -> Result<GeneratedSample, GenerationError> {
    generate_with_config(engine, seed, GenerationConfig::default())
}

/// Generate a sample with custom configuration.
///
/// # Errors
/// Returns `GenerationError` if constraints cannot be satisfied.
pub fn generate_with_config(
    engine: &AstEngine,
    seed: u64,
    config: GenerationConfig,
) -> Result<GeneratedSample, GenerationError> {
    let graph = DependencyGraph::build(engine);
    let order = graph
        .topological_sort()
        .map_err(GenerationError::CycleDetected)?;

    let mut ctx = GenerationContext::new(engine, seed, config);
    let skip_set = ctx.build_skip_set();

    for node_id in &order {
        if skip_set.contains(node_id) {
            continue;
        }
        if let Some(node) = engine.structure.get(*node_id) {
            let kind = node.kind().clone();
            ctx.generate_node_inner(*node_id, &kind)?;
        }
    }

    Ok(ctx.into_sample())
}

// --- Helper functions ---

/// Extract constraints for a node from the engine.
fn get_node_constraints(engine: &AstEngine, node_id: NodeId) -> Vec<&Constraint> {
    let constraint_ids = engine.constraints.for_node(node_id);
    constraint_ids
        .iter()
        .filter_map(|cid| engine.constraints.get(*cid))
        .collect()
}

fn resolve_charset(constraints: &[&Constraint]) -> CharSetSpec {
    for c in constraints {
        if let Constraint::CharSet { charset, .. } = c {
            return charset.clone();
        }
    }
    CharSetSpec::LowerAlpha
}

fn random_char_from_spec(spec: &CharSetSpec, rng: &mut StdRng) -> char {
    match spec {
        CharSetSpec::LowerAlpha => rng.gen_range(b'a'..=b'z') as char,
        CharSetSpec::UpperAlpha => rng.gen_range(b'A'..=b'Z') as char,
        CharSetSpec::Alpha => {
            if rng.gen_bool(0.5) {
                rng.gen_range(b'a'..=b'z') as char
            } else {
                rng.gen_range(b'A'..=b'Z') as char
            }
        }
        CharSetSpec::Digit => rng.gen_range(b'0'..=b'9') as char,
        CharSetSpec::AlphaNumeric => {
            let idx = rng.gen_range(0..62);
            if idx < 26 {
                (b'a' + idx) as char
            } else if idx < 52 {
                (b'A' + (idx - 26)) as char
            } else {
                (b'0' + (idx - 52)) as char
            }
        }
        CharSetSpec::Custom(chars) => {
            if chars.is_empty() {
                'a'
            } else {
                chars[rng.gen_range(0..chars.len())]
            }
        }
        CharSetSpec::Range(lo, hi) => {
            let lo_u32 = u32::from(*lo);
            let hi_u32 = u32::from(*hi);
            let v = rng.gen_range(lo_u32..=hi_u32);
            char::from_u32(v).unwrap_or(*lo)
        }
    }
}

fn generate_property_array(
    tag: &PropertyTag,
    len: usize,
    lo: i64,
    hi: i64,
    rng: &mut StdRng,
) -> Vec<SampleValue> {
    match tag {
        PropertyTag::Permutation => {
            let n = i64::try_from(len).unwrap_or(0);
            let mut perm: Vec<i64> = (1..=n).collect();
            for i in (1..perm.len()).rev() {
                let j = rng.gen_range(0..=i);
                perm.swap(i, j);
            }
            perm.into_iter().map(SampleValue::Int).collect()
        }
        PropertyTag::Tree => {
            if len < 2 {
                return Vec::new();
            }
            let n = len;
            let prufer_len = n.saturating_sub(2);
            let prufer: Vec<usize> = (0..prufer_len).map(|_| rng.gen_range(1..=n)).collect();
            let edges = prufer_to_edges(&prufer, n);
            let mut result = Vec::with_capacity(edges.len() * 2);
            for (u, v) in edges {
                result.push(SampleValue::Int(i64::try_from(u).unwrap_or(0)));
                result.push(SampleValue::Int(i64::try_from(v).unwrap_or(0)));
            }
            result
        }
        PropertyTag::Simple => {
            let n = usize::try_from(hi.min(100)).unwrap_or(10);
            let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
            let target_edges = len;
            let max_attempts = target_edges * 10;
            let mut attempts = 0;
            while edge_set.len() < target_edges && attempts < max_attempts {
                let u = rng.gen_range(1..=n);
                let v = rng.gen_range(1..=n);
                if u != v {
                    let edge = if u < v { (u, v) } else { (v, u) };
                    edge_set.insert(edge);
                }
                attempts += 1;
            }
            let mut result = Vec::with_capacity(edge_set.len() * 2);
            for (u, v) in edge_set {
                result.push(SampleValue::Int(i64::try_from(u).unwrap_or(0)));
                result.push(SampleValue::Int(i64::try_from(v).unwrap_or(0)));
            }
            result
        }
        _ => (0..len)
            .map(|_| SampleValue::Int(rng.gen_range(lo..=hi)))
            .collect(),
    }
}

fn prufer_to_edges(prufer: &[usize], n: usize) -> Vec<(usize, usize)> {
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return Vec::new();
    }
    if n == 2 {
        return vec![(1, 2)];
    }

    let mut degree = vec![1usize; n + 1]; // 1-indexed
    for &p in prufer {
        if p <= n {
            degree[p] += 1;
        }
    }

    let mut edges = Vec::with_capacity(n - 1);
    let mut ptr = 0;

    // Find the first leaf (degree == 1)
    for (i, &deg) in degree.iter().enumerate().skip(1).take(n) {
        if deg == 1 {
            ptr = i;
            break;
        }
    }

    let mut leaf = ptr;
    for &p in prufer {
        edges.push((leaf, p));
        degree[p] -= 1;
        if degree[p] == 1 && p < leaf {
            // p becomes a new leaf and is smaller than current pointer
            leaf = p;
        } else {
            // Advance pointer to next leaf
            ptr += 1;
            while ptr <= n && degree[ptr] != 1 {
                ptr += 1;
            }
            leaf = if ptr <= n { ptr } else { n };
        }
    }

    // The last edge connects the remaining two nodes with degree 1
    // One is `leaf`, the other is `n` (by convention)
    edges.push((leaf, n));
    edges
}

fn sort_sample_values(values: &mut [SampleValue], order: SortOrder) {
    values.sort_by(|a, b| {
        let a_val = match a {
            SampleValue::Int(v) => *v,
            _ => 0,
        };
        let b_val = match b {
            SampleValue::Int(v) => *v,
            _ => 0,
        };
        match order {
            SortOrder::Ascending | SortOrder::NonDecreasing => a_val.cmp(&b_val),
            SortOrder::Descending | SortOrder::NonIncreasing => b_val.cmp(&a_val),
        }
    });
}
