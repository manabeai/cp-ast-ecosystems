use std::collections::HashMap;

use crate::operation::AstEngine;
use crate::structure::{Literal, NodeId, NodeKind, Reference};

use super::generator::{GeneratedSample, SampleValue};

/// Render a generated sample as competitive-programming-style text output.
///
/// Walks the structure tree from the root and emits values in problem input format.
#[must_use]
pub fn sample_to_text(engine: &AstEngine, sample: &GeneratedSample) -> String {
    let mut output = String::new();
    emit_node(engine, engine.structure.root(), sample, &mut output);

    // Remove trailing whitespace on each line, then ensure single trailing newline
    let trimmed: String = output
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");

    let result = trimmed.trim_end().to_owned();
    if result.is_empty() {
        result
    } else {
        result + "\n"
    }
}

fn emit_node(engine: &AstEngine, node_id: NodeId, sample: &GeneratedSample, output: &mut String) {
    let Some(node) = engine.structure.get(node_id) else {
        return;
    };

    match node.kind() {
        NodeKind::Scalar { .. } => {
            if let Some(value) = sample.values.get(&node_id) {
                output.push_str(&format_value(value));
            }
        }
        NodeKind::Array { .. } => {
            if let Some(SampleValue::Array(elements)) = sample.values.get(&node_id) {
                let line: Vec<String> = elements.iter().map(format_value).collect();
                output.push_str(&line.join(" "));
                output.push('\n');
            }
        }
        NodeKind::Matrix { .. } => {
            if let Some(SampleValue::Grid(rows)) = sample.values.get(&node_id) {
                for row in rows {
                    let line: Vec<String> = row.iter().map(format_value).collect();
                    output.push_str(&line.join(" "));
                    output.push('\n');
                }
            }
        }
        NodeKind::Tuple { elements } => {
            // Clone to release borrow on engine
            let elements = elements.clone();
            let mut parts = Vec::new();
            for &child_id in &elements {
                if let Some(value) = sample.values.get(&child_id) {
                    parts.push(format_value(value));
                }
            }
            if !parts.is_empty() {
                output.push_str(&parts.join(" "));
                output.push('\n');
            }
        }
        NodeKind::Sequence { children } => {
            let children = children.clone();
            for &child_id in &children {
                emit_node(engine, child_id, sample, output);
            }
        }
        NodeKind::Section { header, body } => {
            let header = *header;
            let body = body.clone();
            if let Some(h) = header {
                emit_node(engine, h, sample, output);
            }
            for &child_id in &body {
                emit_node(engine, child_id, sample, output);
            }
        }
        NodeKind::Repeat { count: _, body } => {
            let body = body.clone();
            if let Some(instances) = sample.repeat_instances.get(&node_id) {
                for iteration_values in instances {
                    for &child_id in &body {
                        emit_node_with_values(
                            engine,
                            child_id,
                            iteration_values,
                            sample,
                            output,
                        );
                    }
                }
            }
        }
        NodeKind::Choice { tag, variants } => {
            let tag = tag.clone();
            let variants = variants.clone();
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
        NodeKind::Hole { .. } => {
            // Skip holes
        }
    }
}

/// Emit a node using a specific value map (for Repeat iteration rendering).
fn emit_node_with_values(
    engine: &AstEngine,
    node_id: NodeId,
    values: &HashMap<NodeId, SampleValue>,
    sample: &GeneratedSample,
    output: &mut String,
) {
    let Some(node) = engine.structure.get(node_id) else {
        return;
    };

    match node.kind() {
        NodeKind::Scalar { .. } => {
            if let Some(value) = values.get(&node_id) {
                output.push_str(&format_value(value));
            }
        }
        NodeKind::Array { .. } => {
            if let Some(SampleValue::Array(elements)) = values.get(&node_id) {
                let line: Vec<String> = elements.iter().map(format_value).collect();
                output.push_str(&line.join(" "));
                output.push('\n');
            }
        }
        NodeKind::Matrix { .. } => {
            if let Some(SampleValue::Grid(rows)) = values.get(&node_id) {
                for row in rows {
                    let line: Vec<String> = row.iter().map(format_value).collect();
                    output.push_str(&line.join(" "));
                    output.push('\n');
                }
            }
        }
        NodeKind::Tuple { elements } => {
            let elements = elements.clone();
            let mut parts = Vec::new();
            for &child_id in &elements {
                if let Some(value) = values.get(&child_id) {
                    parts.push(format_value(value));
                }
            }
            if !parts.is_empty() {
                output.push_str(&parts.join(" "));
                output.push('\n');
            }
        }
        NodeKind::Sequence { children } => {
            let children = children.clone();
            for &child_id in &children {
                emit_node_with_values(engine, child_id, values, sample, output);
            }
        }
        _ => {
            // For other node types in repeat body, fall back to sample.values
            emit_node(engine, node_id, sample, output);
        }
    }
}

fn literal_matches_value(lit: &Literal, val: &SampleValue) -> bool {
    match (lit, val) {
        (Literal::IntLit(a), SampleValue::Int(b)) => *a == *b,
        (Literal::StrLit(a), SampleValue::Str(b)) => a == b,
        _ => false,
    }
}

fn format_value(value: &SampleValue) -> String {
    match value {
        SampleValue::Int(v) => v.to_string(),
        SampleValue::Str(s) => s.clone(),
        SampleValue::Array(elements) => elements
            .iter()
            .map(format_value)
            .collect::<Vec<_>>()
            .join(" "),
        SampleValue::Grid(rows) => rows
            .iter()
            .map(|row| row.iter().map(format_value).collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}
