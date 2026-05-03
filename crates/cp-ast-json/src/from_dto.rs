//! DTO → Internal conversion for lossless JSON deserialization.

use cp_ast_core::constraint::{
    ArithOp, CharSetSpec, Constraint, ConstraintId, ConstraintSet, DistinctUnit, ExpectedType,
    Expression, PropertyTag, RelationOp, RenderHintKind, Separator, SortOrder,
};
use cp_ast_core::operation::engine::AstEngine;
use cp_ast_core::structure::{
    Ident, Literal, NodeId, NodeKind, NodeKindHint, Reference, StructureAst, StructureNode,
};

use crate::dto::{
    AstDocumentEnvelope, CURRENT_SCHEMA_VERSION, CharSetSpecDto, ChoiceVariantDto, ConstraintDto,
    ConstraintSetDto, ExpressionDto, LiteralDto, NodeKindDto, PropertyTagDto, ReferenceDto,
    RenderHintKindDto, StructureAstDto,
};
use crate::error::ConversionError;

// ── parse helpers ───────────────────────────────────────────────────

fn parse_u64(s: &str) -> Result<u64, ConversionError> {
    s.parse::<u64>()
        .map_err(|_| ConversionError::InvalidId(s.to_owned()))
}

fn parse_node_id(s: &str) -> Result<NodeId, ConversionError> {
    parse_u64(s).map(NodeId::from_raw)
}

fn parse_constraint_id(s: &str) -> Result<ConstraintId, ConversionError> {
    parse_u64(s).map(ConstraintId::from_raw)
}

fn parse_node_ids(ids: &[String]) -> Result<Vec<NodeId>, ConversionError> {
    ids.iter().map(|s| parse_node_id(s)).collect()
}

// ── string → enum parsers ───────────────────────────────────────────

fn str_to_expected_type(s: &str) -> Result<ExpectedType, ConversionError> {
    match s {
        "Int" => Ok(ExpectedType::Int),
        "Str" => Ok(ExpectedType::Str),
        "Char" => Ok(ExpectedType::Char),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "ExpectedType",
            value: s.to_owned(),
        }),
    }
}

fn str_to_relation_op(s: &str) -> Result<RelationOp, ConversionError> {
    match s {
        "Lt" => Ok(RelationOp::Lt),
        "Le" => Ok(RelationOp::Le),
        "Gt" => Ok(RelationOp::Gt),
        "Ge" => Ok(RelationOp::Ge),
        "Eq" => Ok(RelationOp::Eq),
        "Ne" => Ok(RelationOp::Ne),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "RelationOp",
            value: s.to_owned(),
        }),
    }
}

fn str_to_arith_op(s: &str) -> Result<ArithOp, ConversionError> {
    match s {
        "Add" => Ok(ArithOp::Add),
        "Sub" => Ok(ArithOp::Sub),
        "Mul" => Ok(ArithOp::Mul),
        "Div" => Ok(ArithOp::Div),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "ArithOp",
            value: s.to_owned(),
        }),
    }
}

fn str_to_distinct_unit(s: &str) -> Result<DistinctUnit, ConversionError> {
    match s {
        "Element" => Ok(DistinctUnit::Element),
        "Tuple" => Ok(DistinctUnit::Tuple),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "DistinctUnit",
            value: s.to_owned(),
        }),
    }
}

fn str_to_sort_order(s: &str) -> Result<SortOrder, ConversionError> {
    match s {
        "Ascending" => Ok(SortOrder::Ascending),
        "NonDecreasing" => Ok(SortOrder::NonDecreasing),
        "Descending" => Ok(SortOrder::Descending),
        "NonIncreasing" => Ok(SortOrder::NonIncreasing),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "SortOrder",
            value: s.to_owned(),
        }),
    }
}

fn str_to_separator(s: &str) -> Result<Separator, ConversionError> {
    match s {
        "Space" => Ok(Separator::Space),
        "None" => Ok(Separator::None),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "Separator",
            value: s.to_owned(),
        }),
    }
}

fn str_to_hint(s: &str) -> Result<NodeKindHint, ConversionError> {
    match s {
        "AnyScalar" => Ok(NodeKindHint::AnyScalar),
        "AnyArray" => Ok(NodeKindHint::AnyArray),
        "AnyMatrix" => Ok(NodeKindHint::AnyMatrix),
        "AnyTuple" => Ok(NodeKindHint::AnyTuple),
        "AnyRepeat" => Ok(NodeKindHint::AnyRepeat),
        "AnySection" => Ok(NodeKindHint::AnySection),
        "AnyChoice" => Ok(NodeKindHint::AnyChoice),
        "Any" => Ok(NodeKindHint::Any),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "NodeKindHint",
            value: s.to_owned(),
        }),
    }
}

// ── DTO → enum helpers (tagged union DTOs) ──────────────────────────

fn dto_to_property_tag(dto: PropertyTagDto) -> PropertyTag {
    match dto {
        PropertyTagDto::Simple => PropertyTag::Simple,
        PropertyTagDto::Connected => PropertyTag::Connected,
        PropertyTagDto::Tree => PropertyTag::Tree,
        PropertyTagDto::Permutation => PropertyTag::Permutation,
        PropertyTagDto::Binary => PropertyTag::Binary,
        PropertyTagDto::Odd => PropertyTag::Odd,
        PropertyTagDto::Even => PropertyTag::Even,
        PropertyTagDto::Custom { value } => PropertyTag::Custom(value),
    }
}

fn dto_to_charset(dto: CharSetSpecDto) -> CharSetSpec {
    match dto {
        CharSetSpecDto::LowerAlpha => CharSetSpec::LowerAlpha,
        CharSetSpecDto::UpperAlpha => CharSetSpec::UpperAlpha,
        CharSetSpecDto::Alpha => CharSetSpec::Alpha,
        CharSetSpecDto::Digit => CharSetSpec::Digit,
        CharSetSpecDto::AlphaNumeric => CharSetSpec::AlphaNumeric,
        CharSetSpecDto::Custom { chars } => CharSetSpec::Custom(chars),
        CharSetSpecDto::Range { from, to } => CharSetSpec::Range(from, to),
    }
}

fn dto_to_render_hint(dto: RenderHintKindDto) -> Result<RenderHintKind, ConversionError> {
    match dto {
        RenderHintKindDto::Separator { value } => {
            Ok(RenderHintKind::Separator(str_to_separator(&value)?))
        }
    }
}

// ── top-level ───────────────────────────────────────────────────────

/// Convert a deserialised [`AstDocumentEnvelope`] back into an [`AstEngine`].
///
/// # Errors
/// Returns `ConversionError` if the schema version is unsupported, IDs are
/// malformed, or arena slots have mismatched indices.
pub fn envelope_to_engine(envelope: AstDocumentEnvelope) -> Result<AstEngine, ConversionError> {
    if envelope.schema_version != CURRENT_SCHEMA_VERSION {
        return Err(ConversionError::UnsupportedVersion(envelope.schema_version));
    }
    let structure = dto_to_structure(envelope.document.structure)?;
    let constraints = dto_to_constraint_set(envelope.document.constraints)?;
    Ok(AstEngine {
        structure,
        constraints,
    })
}

// ── structure ───────────────────────────────────────────────────────

fn dto_to_structure(dto: StructureAstDto) -> Result<StructureAst, ConversionError> {
    let root = parse_node_id(&dto.root)?;
    let next_id = parse_u64(&dto.next_id)?;

    let mut arena: Vec<Option<StructureNode>> = Vec::with_capacity(dto.arena.len());
    for (index, slot) in dto.arena.into_iter().enumerate() {
        match slot {
            None => arena.push(None),
            Some(node_dto) => {
                let id = parse_node_id(&node_dto.id)?;
                let expected_index = u64::try_from(index)
                    .map_err(|_| ConversionError::InvalidId(index.to_string()))?;
                if id.value() != expected_index {
                    return Err(ConversionError::IdIndexMismatch {
                        expected: expected_index,
                        actual: id.value(),
                    });
                }
                let kind = dto_to_node_kind(node_dto.kind)?;
                arena.push(Some(StructureNode::new(id, kind)));
            }
        }
    }

    Ok(StructureAst::from_raw_parts(root, arena, next_id))
}

// ── node kind ───────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
fn dto_to_node_kind(dto: NodeKindDto) -> Result<NodeKind, ConversionError> {
    match dto {
        NodeKindDto::Scalar { name } => Ok(NodeKind::Scalar {
            name: Ident::new(&name),
        }),
        NodeKindDto::Array { name, length } => Ok(NodeKind::Array {
            name: Ident::new(&name),
            length: dto_to_expr(length)?,
        }),
        NodeKindDto::Matrix { name, rows, cols } => Ok(NodeKind::Matrix {
            name: Ident::new(&name),
            rows: dto_to_ref(rows)?,
            cols: dto_to_ref(cols)?,
        }),
        NodeKindDto::Tuple { elements } => Ok(NodeKind::Tuple {
            elements: parse_node_ids(&elements)?,
        }),
        NodeKindDto::Repeat {
            count,
            index_var,
            body,
        } => Ok(NodeKind::Repeat {
            count: dto_to_expr(count)?,
            index_var: index_var.map(|s| Ident::new(&s)),
            body: parse_node_ids(&body)?,
        }),
        NodeKindDto::Section { header, body } => Ok(NodeKind::Section {
            header: header.map(|s| parse_node_id(&s)).transpose()?,
            body: parse_node_ids(&body)?,
        }),
        NodeKindDto::Sequence { children } => Ok(NodeKind::Sequence {
            children: parse_node_ids(&children)?,
        }),
        NodeKindDto::Choice { tag, variants } => {
            let converted_variants = variants
                .into_iter()
                .map(|v: ChoiceVariantDto| {
                    let lit = dto_to_literal(v.tag_value);
                    let body = parse_node_ids(&v.body)?;
                    Ok((lit, body))
                })
                .collect::<Result<Vec<_>, ConversionError>>()?;
            Ok(NodeKind::Choice {
                tag: dto_to_ref(tag)?,
                variants: converted_variants,
            })
        }
        NodeKindDto::Hole { expected_kind } => Ok(NodeKind::Hole {
            expected_kind: expected_kind.map(|s| str_to_hint(&s)).transpose()?,
        }),
    }
}

// ── constraint set ──────────────────────────────────────────────────

fn dto_to_constraint_set(dto: ConstraintSetDto) -> Result<ConstraintSet, ConversionError> {
    let next_id = parse_u64(&dto.next_id)?;

    let mut arena: Vec<Option<Constraint>> = Vec::with_capacity(dto.arena.len());
    for (index, slot) in dto.arena.into_iter().enumerate() {
        match slot {
            None => arena.push(None),
            Some(entry) => {
                let id = parse_constraint_id(&entry.id)?;
                let expected_index = u64::try_from(index)
                    .map_err(|_| ConversionError::InvalidId(index.to_string()))?;
                if id.value() != expected_index {
                    return Err(ConversionError::IdIndexMismatch {
                        expected: expected_index,
                        actual: id.value(),
                    });
                }
                arena.push(Some(dto_to_constraint(entry.constraint)?));
            }
        }
    }

    let by_node = dto
        .by_node
        .into_iter()
        .map(|entry| {
            let node_id = parse_node_id(&entry.node_id)?;
            let cids = entry
                .constraints
                .iter()
                .map(|s| parse_constraint_id(s))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((node_id, cids))
        })
        .collect::<Result<Vec<_>, ConversionError>>()?;

    let global = dto
        .global
        .iter()
        .map(|s| parse_constraint_id(s))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ConstraintSet::from_raw_parts(
        arena, by_node, global, next_id,
    ))
}

// ── constraint ──────────────────────────────────────────────────────

#[allow(clippy::too_many_lines)]
fn dto_to_constraint(dto: ConstraintDto) -> Result<Constraint, ConversionError> {
    match dto {
        ConstraintDto::Range {
            target,
            lower,
            upper,
        } => Ok(Constraint::Range {
            target: dto_to_ref(target)?,
            lower: dto_to_expr(lower)?,
            upper: dto_to_expr(upper)?,
        }),
        ConstraintDto::TypeDecl { target, expected } => Ok(Constraint::TypeDecl {
            target: dto_to_ref(target)?,
            expected: str_to_expected_type(&expected)?,
        }),
        ConstraintDto::LengthRelation { target, length } => Ok(Constraint::LengthRelation {
            target: dto_to_ref(target)?,
            length: dto_to_expr(length)?,
        }),
        ConstraintDto::Relation { lhs, op, rhs } => Ok(Constraint::Relation {
            lhs: dto_to_expr(lhs)?,
            op: str_to_relation_op(&op)?,
            rhs: dto_to_expr(rhs)?,
        }),
        ConstraintDto::Distinct { elements, unit } => Ok(Constraint::Distinct {
            elements: dto_to_ref(elements)?,
            unit: str_to_distinct_unit(&unit)?,
        }),
        ConstraintDto::Property { target, tag } => Ok(Constraint::Property {
            target: dto_to_ref(target)?,
            tag: dto_to_property_tag(tag),
        }),
        ConstraintDto::SumBound { variable, upper } => Ok(Constraint::SumBound {
            variable: dto_to_ref(variable)?,
            upper: dto_to_expr(upper)?,
        }),
        ConstraintDto::Sorted { elements, order } => Ok(Constraint::Sorted {
            elements: dto_to_ref(elements)?,
            order: str_to_sort_order(&order)?,
        }),
        ConstraintDto::Guarantee {
            description,
            predicate,
        } => Ok(Constraint::Guarantee {
            description,
            predicate: predicate.map(dto_to_expr).transpose()?,
        }),
        ConstraintDto::CharSet { target, charset } => Ok(Constraint::CharSet {
            target: dto_to_ref(target)?,
            charset: dto_to_charset(charset),
        }),
        ConstraintDto::StringLength { target, min, max } => Ok(Constraint::StringLength {
            target: dto_to_ref(target)?,
            min: dto_to_expr(min)?,
            max: dto_to_expr(max)?,
        }),
        ConstraintDto::RenderHint { target, hint } => Ok(Constraint::RenderHint {
            target: dto_to_ref(target)?,
            hint: dto_to_render_hint(hint)?,
        }),
    }
}

// ── expression ──────────────────────────────────────────────────────

fn dto_to_expr(dto: ExpressionDto) -> Result<Expression, ConversionError> {
    match dto {
        ExpressionDto::Lit { value } => Ok(Expression::Lit(value)),
        ExpressionDto::Var { reference } => Ok(Expression::Var(dto_to_ref(reference)?)),
        ExpressionDto::BinOp { op, lhs, rhs } => Ok(Expression::BinOp {
            op: str_to_arith_op(&op)?,
            lhs: Box::new(dto_to_expr(*lhs)?),
            rhs: Box::new(dto_to_expr(*rhs)?),
        }),
        ExpressionDto::Pow { base, exp } => Ok(Expression::Pow {
            base: Box::new(dto_to_expr(*base)?),
            exp: Box::new(dto_to_expr(*exp)?),
        }),
        ExpressionDto::FnCall { name, args } => Ok(Expression::FnCall {
            name: Ident::new(&name),
            args: args
                .into_iter()
                .map(dto_to_expr)
                .collect::<Result<Vec<_>, _>>()?,
        }),
    }
}

// ── reference ───────────────────────────────────────────────────────

fn dto_to_ref(dto: ReferenceDto) -> Result<Reference, ConversionError> {
    match dto {
        ReferenceDto::VariableRef { node_id } => {
            Ok(Reference::VariableRef(parse_node_id(&node_id)?))
        }
        ReferenceDto::IndexedRef { target, indices } => Ok(Reference::IndexedRef {
            target: parse_node_id(&target)?,
            indices: indices.iter().map(|s| Ident::new(s)).collect(),
        }),
        ReferenceDto::Unresolved { name } => Ok(Reference::Unresolved(Ident::new(&name))),
    }
}

// ── literal ─────────────────────────────────────────────────────────

fn dto_to_literal(dto: LiteralDto) -> Literal {
    match dto {
        LiteralDto::IntLit { value } => Literal::IntLit(value),
        LiteralDto::StrLit { value } => Literal::StrLit(value),
    }
}
