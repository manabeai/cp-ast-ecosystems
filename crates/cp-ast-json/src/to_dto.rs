//! Internal → DTO conversion for lossless JSON serialization.

use cp_ast_core::constraint::{
    ArithOp, CharSetSpec, Constraint, ConstraintId, ConstraintSet, DistinctUnit, ExpectedType,
    Expression, PropertyTag, RelationOp, RenderHintKind, Separator, SortOrder,
};
use cp_ast_core::operation::engine::AstEngine;
use cp_ast_core::structure::{
    Ident, Literal, NodeId, NodeKind, NodeKindHint, Reference, StructureAst, StructureNode,
};

use crate::dto::{
    AstDocumentDto, AstDocumentEnvelope, ByNodeEntryDto, CURRENT_SCHEMA_VERSION, CharSetSpecDto,
    ChoiceVariantDto, ConstraintDto, ConstraintEntryDto, ConstraintSetDto, ExpressionDto,
    LiteralDto, NodeKindDto, PropertyTagDto, ReferenceDto, RenderHintKindDto, StructureAstDto,
    StructureNodeDto,
};

// ── helpers ─────────────────────────────────────────────────────────

fn node_id_str(id: NodeId) -> String {
    id.value().to_string()
}

fn constraint_id_str(id: ConstraintId) -> String {
    id.value().to_string()
}

fn node_ids_to_strings(ids: &[NodeId]) -> Vec<String> {
    ids.iter().copied().map(node_id_str).collect()
}

// ── top-level ───────────────────────────────────────────────────────

/// Convert an [`AstEngine`] snapshot into a serialisable envelope.
#[must_use]
pub fn engine_to_envelope(engine: &AstEngine) -> AstDocumentEnvelope {
    AstDocumentEnvelope {
        schema_version: CURRENT_SCHEMA_VERSION,
        document: AstDocumentDto {
            structure: structure_to_dto(&engine.structure),
            constraints: constraint_set_to_dto(&engine.constraints),
        },
    }
}

// ── structure ───────────────────────────────────────────────────────

fn structure_to_dto(ast: &StructureAst) -> StructureAstDto {
    StructureAstDto {
        root: node_id_str(ast.root()),
        next_id: ast.next_id().to_string(),
        arena: ast
            .arena_raw()
            .iter()
            .map(|slot| slot.as_ref().map(structure_node_to_dto))
            .collect(),
    }
}

fn structure_node_to_dto(node: &StructureNode) -> StructureNodeDto {
    StructureNodeDto {
        id: node_id_str(node.id()),
        kind: node_kind_to_dto(node.kind()),
    }
}

fn node_kind_to_dto(kind: &NodeKind) -> NodeKindDto {
    match kind {
        NodeKind::Scalar { name } => NodeKindDto::Scalar {
            name: ident_str(name),
        },
        NodeKind::Array { name, length } => NodeKindDto::Array {
            name: ident_str(name),
            length: expr_to_dto(length),
        },
        NodeKind::Matrix { name, rows, cols } => NodeKindDto::Matrix {
            name: ident_str(name),
            rows: ref_to_dto(rows),
            cols: ref_to_dto(cols),
        },
        NodeKind::Tuple { elements } => NodeKindDto::Tuple {
            elements: node_ids_to_strings(elements),
        },
        NodeKind::Repeat {
            count,
            index_var,
            body,
        } => NodeKindDto::Repeat {
            count: expr_to_dto(count),
            index_var: index_var.as_ref().map(ident_str),
            body: node_ids_to_strings(body),
        },
        NodeKind::Section { header, body } => NodeKindDto::Section {
            header: header.map(node_id_str),
            body: node_ids_to_strings(body),
        },
        NodeKind::Sequence { children } => NodeKindDto::Sequence {
            children: node_ids_to_strings(children),
        },
        NodeKind::Choice { tag, variants } => NodeKindDto::Choice {
            tag: ref_to_dto(tag),
            variants: variants
                .iter()
                .map(|(lit, body)| ChoiceVariantDto {
                    tag_value: literal_to_dto(lit),
                    body: node_ids_to_strings(body),
                })
                .collect(),
        },
        NodeKind::Hole { expected_kind } => NodeKindDto::Hole {
            expected_kind: expected_kind.map(node_kind_hint_str),
        },
    }
}

// ── constraints ─────────────────────────────────────────────────────

fn constraint_set_to_dto(cs: &ConstraintSet) -> ConstraintSetDto {
    ConstraintSetDto {
        next_id: cs.next_id().to_string(),
        arena: cs
            .arena_raw()
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                slot.as_ref().map(|c| ConstraintEntryDto {
                    id: i.to_string(),
                    constraint: constraint_to_dto(c),
                })
            })
            .collect(),
        by_node: cs
            .by_node_raw()
            .iter()
            .map(|(node_id, cids)| ByNodeEntryDto {
                node_id: node_id_str(*node_id),
                constraints: cids.iter().copied().map(constraint_id_str).collect(),
            })
            .collect(),
        global: cs.global().iter().copied().map(constraint_id_str).collect(),
    }
}

fn constraint_to_dto(c: &Constraint) -> ConstraintDto {
    match c {
        Constraint::Range {
            target,
            lower,
            upper,
        } => ConstraintDto::Range {
            target: ref_to_dto(target),
            lower: expr_to_dto(lower),
            upper: expr_to_dto(upper),
        },
        Constraint::TypeDecl { target, expected } => ConstraintDto::TypeDecl {
            target: ref_to_dto(target),
            expected: expected_type_str(expected),
        },
        Constraint::LengthRelation { target, length } => ConstraintDto::LengthRelation {
            target: ref_to_dto(target),
            length: expr_to_dto(length),
        },
        Constraint::Relation { lhs, op, rhs } => ConstraintDto::Relation {
            lhs: expr_to_dto(lhs),
            op: relation_op_str(*op),
            rhs: expr_to_dto(rhs),
        },
        Constraint::Distinct { elements, unit } => ConstraintDto::Distinct {
            elements: ref_to_dto(elements),
            unit: distinct_unit_str(unit),
        },
        Constraint::Property { target, tag } => ConstraintDto::Property {
            target: ref_to_dto(target),
            tag: property_tag_to_dto(tag),
        },
        Constraint::SumBound { variable, upper } => ConstraintDto::SumBound {
            variable: ref_to_dto(variable),
            upper: expr_to_dto(upper),
        },
        Constraint::Sorted { elements, order } => ConstraintDto::Sorted {
            elements: ref_to_dto(elements),
            order: sort_order_str(*order),
        },
        Constraint::Guarantee {
            description,
            predicate,
        } => ConstraintDto::Guarantee {
            description: description.clone(),
            predicate: predicate.as_ref().map(expr_to_dto),
        },
        Constraint::CharSet { target, charset } => ConstraintDto::CharSet {
            target: ref_to_dto(target),
            charset: charset_spec_to_dto(charset),
        },
        Constraint::StringLength { target, min, max } => ConstraintDto::StringLength {
            target: ref_to_dto(target),
            min: expr_to_dto(min),
            max: expr_to_dto(max),
        },
        Constraint::RenderHint { target, hint } => ConstraintDto::RenderHint {
            target: ref_to_dto(target),
            hint: render_hint_kind_to_dto(hint),
        },
    }
}

// ── expressions ─────────────────────────────────────────────────────

fn expr_to_dto(e: &Expression) -> ExpressionDto {
    match e {
        Expression::Lit(v) => ExpressionDto::Lit { value: *v },
        Expression::Var(r) => ExpressionDto::Var {
            reference: ref_to_dto(r),
        },
        Expression::BinOp { op, lhs, rhs } => ExpressionDto::BinOp {
            op: arith_op_str(*op),
            lhs: Box::new(expr_to_dto(lhs)),
            rhs: Box::new(expr_to_dto(rhs)),
        },
        Expression::Pow { base, exp } => ExpressionDto::Pow {
            base: Box::new(expr_to_dto(base)),
            exp: Box::new(expr_to_dto(exp)),
        },
        Expression::FnCall { name, args } => ExpressionDto::FnCall {
            name: ident_str(name),
            args: args.iter().map(expr_to_dto).collect(),
        },
    }
}

// ── references ──────────────────────────────────────────────────────

fn ref_to_dto(r: &Reference) -> ReferenceDto {
    match r {
        Reference::VariableRef(id) => ReferenceDto::VariableRef {
            node_id: node_id_str(*id),
        },
        Reference::IndexedRef { target, indices } => ReferenceDto::IndexedRef {
            target: node_id_str(*target),
            indices: indices.iter().map(ident_str).collect(),
        },
        Reference::Unresolved(name) => ReferenceDto::Unresolved {
            name: ident_str(name),
        },
    }
}

// ── literal ─────────────────────────────────────────────────────────

fn literal_to_dto(l: &Literal) -> LiteralDto {
    match l {
        Literal::IntLit(v) => LiteralDto::IntLit { value: *v },
        Literal::StrLit(s) => LiteralDto::StrLit { value: s.clone() },
    }
}

// ── small-enum string helpers ───────────────────────────────────────

fn ident_str(id: &Ident) -> String {
    id.as_str().to_owned()
}

fn expected_type_str(et: &ExpectedType) -> String {
    match et {
        ExpectedType::Int => "Int".to_owned(),
        ExpectedType::Str => "Str".to_owned(),
        ExpectedType::Char => "Char".to_owned(),
    }
}

fn relation_op_str(op: RelationOp) -> String {
    match op {
        RelationOp::Lt => "Lt".to_owned(),
        RelationOp::Le => "Le".to_owned(),
        RelationOp::Gt => "Gt".to_owned(),
        RelationOp::Ge => "Ge".to_owned(),
        RelationOp::Eq => "Eq".to_owned(),
        RelationOp::Ne => "Ne".to_owned(),
    }
}

fn arith_op_str(op: ArithOp) -> String {
    match op {
        ArithOp::Add => "Add".to_owned(),
        ArithOp::Sub => "Sub".to_owned(),
        ArithOp::Mul => "Mul".to_owned(),
        ArithOp::Div => "Div".to_owned(),
    }
}

fn distinct_unit_str(u: &DistinctUnit) -> String {
    match *u {
        DistinctUnit::Element => "Element".to_owned(),
        DistinctUnit::Tuple => "Tuple".to_owned(),
    }
}

fn sort_order_str(o: SortOrder) -> String {
    match o {
        SortOrder::Ascending => "Ascending".to_owned(),
        SortOrder::NonDecreasing => "NonDecreasing".to_owned(),
        SortOrder::Descending => "Descending".to_owned(),
        SortOrder::NonIncreasing => "NonIncreasing".to_owned(),
    }
}

fn node_kind_hint_str(h: NodeKindHint) -> String {
    match h {
        NodeKindHint::AnyScalar => "AnyScalar".to_owned(),
        NodeKindHint::AnyArray => "AnyArray".to_owned(),
        NodeKindHint::AnyMatrix => "AnyMatrix".to_owned(),
        NodeKindHint::AnyTuple => "AnyTuple".to_owned(),
        NodeKindHint::AnyRepeat => "AnyRepeat".to_owned(),
        NodeKindHint::AnySection => "AnySection".to_owned(),
        NodeKindHint::AnyChoice => "AnyChoice".to_owned(),
        NodeKindHint::Any => "Any".to_owned(),
    }
}

fn separator_str(s: Separator) -> String {
    match s {
        Separator::Space => "Space".to_owned(),
        Separator::None => "None".to_owned(),
    }
}

// ── small-enum DTO helpers ──────────────────────────────────────────

fn property_tag_to_dto(tag: &PropertyTag) -> PropertyTagDto {
    match tag {
        PropertyTag::Simple => PropertyTagDto::Simple,
        PropertyTag::Connected => PropertyTagDto::Connected,
        PropertyTag::Tree => PropertyTagDto::Tree,
        PropertyTag::Permutation => PropertyTagDto::Permutation,
        PropertyTag::Binary => PropertyTagDto::Binary,
        PropertyTag::Odd => PropertyTagDto::Odd,
        PropertyTag::Even => PropertyTagDto::Even,
        PropertyTag::Custom(v) => PropertyTagDto::Custom { value: v.clone() },
    }
}

fn charset_spec_to_dto(cs: &CharSetSpec) -> CharSetSpecDto {
    match cs {
        CharSetSpec::LowerAlpha => CharSetSpecDto::LowerAlpha,
        CharSetSpec::UpperAlpha => CharSetSpecDto::UpperAlpha,
        CharSetSpec::Alpha => CharSetSpecDto::Alpha,
        CharSetSpec::Digit => CharSetSpecDto::Digit,
        CharSetSpec::AlphaNumeric => CharSetSpecDto::AlphaNumeric,
        CharSetSpec::Custom(chars) => CharSetSpecDto::Custom {
            chars: chars.clone(),
        },
        CharSetSpec::Range(from, to) => CharSetSpecDto::Range {
            from: *from,
            to: *to,
        },
    }
}

fn render_hint_kind_to_dto(hint: &RenderHintKind) -> RenderHintKindDto {
    match hint {
        RenderHintKind::Separator(sep) => RenderHintKindDto::Separator {
            value: separator_str(*sep),
        },
    }
}
