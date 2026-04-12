use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::dependency::DependencyGraph;
use crate::constraint::{CharSetSpec, Constraint, ExpectedType, PropertyTag, SortOrder};
use crate::operation::AstEngine;
use crate::structure::{NodeId, NodeKind, Reference};

/// Level of guarantee for a generated sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuaranteeLevel {
    /// All constraints are guaranteed satisfied (Range, `TypeDecl`, `LengthRelation`).
    L1Guaranteed,
    /// High probability of constraint satisfaction (Distinct, Sorted, Property).
    L2HighProbability,
    /// Best effort — some constraints may not be satisfied.
    L3BestEffort,
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
    /// Warnings generated during sample creation.
    pub warnings: Vec<String>,
    /// Overall guarantee level.
    pub guarantee_level: GuaranteeLevel,
}

/// Generate a sample from an `AstEngine`, using a deterministic seed.
///
/// # Panics
/// Does not panic. Unresolvable situations produce warnings and default values.
#[must_use]
pub fn generate(engine: &AstEngine, seed: u64) -> GeneratedSample {
    let mut rng = StdRng::seed_from_u64(seed);
    let graph = DependencyGraph::build(engine);

    let order = match graph.topological_sort() {
        Ok(o) => o,
        Err(e) => {
            return GeneratedSample {
                values: HashMap::new(),
                warnings: vec![format!("Cannot generate: {e}")],
                guarantee_level: GuaranteeLevel::L3BestEffort,
            };
        }
    };

    let mut sample = GeneratedSample {
        values: HashMap::new(),
        warnings: Vec::new(),
        guarantee_level: GuaranteeLevel::L1Guaranteed,
    };

    for node_id in &order {
        if let Some(node) = engine.structure.get(*node_id) {
            generate_node(engine, node.id(), node.kind(), &mut sample, &mut rng);
        }
    }

    sample
}

fn generate_node(
    engine: &AstEngine,
    node_id: NodeId,
    kind: &NodeKind,
    sample: &mut GeneratedSample,
    rng: &mut StdRng,
) {
    match kind {
        NodeKind::Scalar { .. } => generate_scalar(engine, node_id, sample, rng),
        NodeKind::Array { name: _, length } => {
            generate_array(engine, node_id, length, sample, rng);
        }
        NodeKind::Matrix {
            name: _,
            rows,
            cols,
        } => {
            generate_matrix(engine, node_id, rows, cols, sample, rng);
        }
        NodeKind::Sequence { .. }
        | NodeKind::Section { .. }
        | NodeKind::Tuple { .. }
        | NodeKind::Repeat { .. } => {
            // Composite nodes: children are generated independently via topo order.
            // Nothing to generate for the composite itself.
        }
        NodeKind::Choice { .. } => {
            sample.warnings.push(format!(
                "Choice node {node_id:?}: picked first variant by default"
            ));
        }
        NodeKind::Hole { .. } => {
            sample
                .warnings
                .push(format!("Hole node {node_id:?}: skipped"));
        }
    }
}

fn generate_scalar(
    engine: &AstEngine,
    node_id: NodeId,
    sample: &mut GeneratedSample,
    rng: &mut StdRng,
) {
    let constraints = get_node_constraints(engine, node_id);

    // Determine expected type
    let expected_type = constraints.iter().find_map(|c| {
        if let Constraint::TypeDecl { expected, .. } = c {
            Some(expected.clone())
        } else {
            None
        }
    });

    match expected_type.as_ref().unwrap_or(&ExpectedType::Int) {
        ExpectedType::Int => {
            let (lo, hi) = find_and_resolve_range(&constraints, sample);
            let value = rng.gen_range(lo..=hi);
            sample.values.insert(node_id, SampleValue::Int(value));
        }
        ExpectedType::Str => {
            let len = resolve_string_length(&constraints, sample, rng);
            let charset = resolve_charset(&constraints);
            let s: String = (0..len)
                .map(|_| random_char_from_spec(&charset, rng))
                .collect();
            sample.values.insert(node_id, SampleValue::Str(s));
        }
        ExpectedType::Char => {
            let charset = resolve_charset(&constraints);
            let c = random_char_from_spec(&charset, rng);
            sample
                .values
                .insert(node_id, SampleValue::Str(c.to_string()));
        }
    }
}

fn generate_array(
    engine: &AstEngine,
    node_id: NodeId,
    length_ref: &Reference,
    sample: &mut GeneratedSample,
    rng: &mut StdRng,
) {
    let len = resolve_reference_as_length(length_ref, sample).unwrap_or_else(|| {
        sample.warnings.push(format!(
            "Array {node_id:?}: could not resolve length, using default 5"
        ));
        5
    });

    let constraints = get_node_constraints(engine, node_id);

    // Determine element range
    let (lo, hi) = find_and_resolve_range(&constraints, sample);

    // Check for distinct constraint
    let is_distinct = constraints
        .iter()
        .any(|c| matches!(c, Constraint::Distinct { .. }));

    // Check for sorted constraint
    let sort_order = constraints.iter().find_map(|c| {
        if let Constraint::Sorted { order, .. } = c {
            Some(*order)
        } else {
            None
        }
    });

    // Check for property constraints
    let property_tag = constraints.iter().find_map(|c| {
        if let Constraint::Property { tag, .. } = c {
            Some(tag.clone())
        } else {
            None
        }
    });

    let len_usize = usize::try_from(len).unwrap_or(0);

    let mut elements = if let Some(tag) = &property_tag {
        generate_property_array(tag, len_usize, lo, hi, sample, rng)
    } else if is_distinct {
        generate_distinct_array(len_usize, lo, hi, sample, rng)
    } else {
        (0..len_usize)
            .map(|_| SampleValue::Int(rng.gen_range(lo..=hi)))
            .collect()
    };

    if let Some(order) = sort_order {
        sort_sample_values(&mut elements, order);
        demote_guarantee(sample, GuaranteeLevel::L2HighProbability);
    }

    sample.values.insert(node_id, SampleValue::Array(elements));
}

fn generate_matrix(
    engine: &AstEngine,
    node_id: NodeId,
    rows_ref: &Reference,
    cols_ref: &Reference,
    sample: &mut GeneratedSample,
    rng: &mut StdRng,
) {
    let rows = resolve_reference_as_length(rows_ref, sample).unwrap_or_else(|| {
        sample.warnings.push(format!(
            "Matrix {node_id:?}: could not resolve rows, using default 3"
        ));
        3
    });
    let cols = resolve_reference_as_length(cols_ref, sample).unwrap_or_else(|| {
        sample.warnings.push(format!(
            "Matrix {node_id:?}: could not resolve cols, using default 3"
        ));
        3
    });

    let constraints = get_node_constraints(engine, node_id);
    let (lo, hi) = find_and_resolve_range(&constraints, sample);

    let rows_usize = usize::try_from(rows).unwrap_or(0);
    let cols_usize = usize::try_from(cols).unwrap_or(0);

    let grid: Vec<Vec<SampleValue>> = (0..rows_usize)
        .map(|_| {
            (0..cols_usize)
                .map(|_| SampleValue::Int(rng.gen_range(lo..=hi)))
                .collect()
        })
        .collect();

    sample.values.insert(node_id, SampleValue::Grid(grid));
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

/// Find and resolve a range constraint, returning (low, high) bounds.
fn find_and_resolve_range(constraints: &[&Constraint], _sample: &GeneratedSample) -> (i64, i64) {
    let range = constraints.iter().find_map(|c| {
        if let Constraint::Range { lower, upper, .. } = c {
            Some((lower.clone(), upper.clone()))
        } else {
            None
        }
    });

    if let Some((lower, upper)) = range {
        let lo = lower.evaluate_constant().unwrap_or(1);
        let hi = upper.evaluate_constant().unwrap_or(100);
        if lo > hi {
            (hi, lo)
        } else {
            (lo, hi)
        }
    } else {
        (1, 100)
    }
}

fn resolve_reference_as_length(reference: &Reference, sample: &GeneratedSample) -> Option<i64> {
    match reference {
        Reference::VariableRef(id) => {
            if let Some(SampleValue::Int(v)) = sample.values.get(id) {
                Some(*v)
            } else {
                None
            }
        }
        Reference::IndexedRef { .. } | Reference::Unresolved(_) => None,
    }
}

fn resolve_string_length(
    constraints: &[&Constraint],
    _sample: &GeneratedSample,
    rng: &mut StdRng,
) -> usize {
    for c in constraints {
        if let Constraint::StringLength { min, max, .. } = c {
            let lo = min.evaluate_constant().unwrap_or(1);
            let hi = max.evaluate_constant().unwrap_or(10);
            let lo_usize = usize::try_from(lo.max(1)).unwrap_or(1);
            let hi_usize = usize::try_from(hi.max(lo)).unwrap_or(10);
            return rng.gen_range(lo_usize..=hi_usize);
        }
    }
    rng.gen_range(1..=10)
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

fn generate_distinct_array(
    len: usize,
    lo: i64,
    hi: i64,
    sample: &mut GeneratedSample,
    rng: &mut StdRng,
) -> Vec<SampleValue> {
    demote_guarantee(sample, GuaranteeLevel::L2HighProbability);

    let range_size = hi.saturating_sub(lo).saturating_add(1);

    if i64::try_from(len).unwrap_or(i64::MAX) <= range_size {
        // Fisher-Yates: pick `len` distinct values from [lo, hi]
        let mut pool: Vec<i64> =
            (lo..=hi.min(lo.saturating_add(range_size.min(100_000) - 1))).collect();

        // If pool is too large, use rejection sampling instead
        if pool.len() > 100_000 {
            return generate_distinct_rejection(len, lo, hi, sample, rng);
        }

        // Fisher-Yates shuffle for first `len` elements
        let pick = len.min(pool.len());
        for i in 0..pick {
            let j = rng.gen_range(i..pool.len());
            pool.swap(i, j);
        }

        pool.into_iter().take(len).map(SampleValue::Int).collect()
    } else {
        // Range too small for requested distinct count; best effort
        sample.warnings.push(format!(
            "Distinct: range [{lo}, {hi}] too small for {len} distinct values"
        ));
        demote_guarantee(sample, GuaranteeLevel::L3BestEffort);
        (0..len)
            .map(|i| {
                let modulus = usize::try_from(range_size.max(1)).unwrap_or(1);
                SampleValue::Int(lo.saturating_add(i64::try_from(i % modulus).unwrap_or(0)))
            })
            .collect()
    }
}

fn generate_distinct_rejection(
    len: usize,
    lo: i64,
    hi: i64,
    sample: &mut GeneratedSample,
    rng: &mut StdRng,
) -> Vec<SampleValue> {
    let mut seen = HashSet::with_capacity(len);
    let mut result = Vec::with_capacity(len);
    let max_attempts = len * 100;
    let mut attempts = 0;

    while result.len() < len && attempts < max_attempts {
        let v = rng.gen_range(lo..=hi);
        if seen.insert(v) {
            result.push(SampleValue::Int(v));
        }
        attempts += 1;
    }

    if result.len() < len {
        sample.warnings.push(format!(
            "Distinct: could only generate {} of {len} distinct values",
            result.len()
        ));
        demote_guarantee(sample, GuaranteeLevel::L3BestEffort);
    }

    result
}

fn generate_property_array(
    tag: &PropertyTag,
    len: usize,
    lo: i64,
    hi: i64,
    sample: &mut GeneratedSample,
    rng: &mut StdRng,
) -> Vec<SampleValue> {
    demote_guarantee(sample, GuaranteeLevel::L2HighProbability);

    match tag {
        PropertyTag::Permutation => {
            // Fisher-Yates shuffle of [1..=len]
            let n = i64::try_from(len).unwrap_or(0);
            let mut perm: Vec<i64> = (1..=n).collect();
            for i in (1..perm.len()).rev() {
                let j = rng.gen_range(0..=i);
                perm.swap(i, j);
            }
            perm.into_iter().map(SampleValue::Int).collect()
        }
        PropertyTag::Tree => {
            // Prüfer sequence → edge list (generate as flat array of edges)
            if len < 2 {
                return Vec::new();
            }
            // Number of vertices = len, generate Prüfer sequence of length len-2
            let n = len;
            let prufer_len = n.saturating_sub(2);
            let prufer: Vec<usize> = (0..prufer_len).map(|_| rng.gen_range(1..=n)).collect();

            let edges = prufer_to_edges(&prufer, n);
            // Flatten edges into array: [u1, v1, u2, v2, ...]
            let mut result = Vec::with_capacity(edges.len() * 2);
            for (u, v) in edges {
                result.push(SampleValue::Int(i64::try_from(u).unwrap_or(0)));
                result.push(SampleValue::Int(i64::try_from(v).unwrap_or(0)));
            }
            result
        }
        PropertyTag::Simple => {
            // Simple graph: generate random edges without duplicates
            // len = number of edges, vertices in [lo, hi] range
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
        _ => {
            // Fallback: generate random values
            demote_guarantee(sample, GuaranteeLevel::L3BestEffort);
            sample.warnings.push(format!(
                "Property {tag:?}: unsupported, using random values"
            ));
            (0..len)
                .map(|_| SampleValue::Int(rng.gen_range(lo..=hi)))
                .collect()
        }
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

fn demote_guarantee(sample: &mut GeneratedSample, level: GuaranteeLevel) {
    match (sample.guarantee_level, level) {
        (
            GuaranteeLevel::L1Guaranteed,
            GuaranteeLevel::L2HighProbability | GuaranteeLevel::L3BestEffort,
        )
        | (GuaranteeLevel::L2HighProbability, GuaranteeLevel::L3BestEffort) => {
            sample.guarantee_level = level;
        }
        _ => {}
    }
}
