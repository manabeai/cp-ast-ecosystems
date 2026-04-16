//! Internal → DTO conversion for lossless JSON serialization.

use cp_ast_core::constraint::{
    ArithOp, CharSetSpec, Constraint, ConstraintId, ConstraintSet, DistinctUnit, ExpectedType,
    Expression, PropertyTag, RelationOp, RenderHintKind, Separator, SortOrder,
};
use cp_ast_core::operation::{
    engine::AstEngine,
    error::{OperationError, ViolationDetail},
    result::ApplyResult,
    Action, ConstraintDef, ConstraintDefKind, FillContent, LengthSpec, SlotId, SumBoundDef,
    VarType,
};
use cp_ast_core::projection::{
    CompletenessInfo, ConstraintSummary, ConstraintTargetMenu, Diagnostic, DiagnosticLevel,
    ExprCandidateMenu, FullProjection, HoleCandidate, NodeDetailProjection, OutlineNode,
    ReferenceCandidate, SlotInfo,
};
use cp_ast_core::structure::{
    Ident, Literal, NodeId, NodeKind, NodeKindHint, Reference, StructureAst, StructureNode,
};

use crate::dto::{
    ActionDto,
    ApplyResultDto,
    AstDocumentDto,
    AstDocumentEnvelope,
    ByNodeEntryDto,
    CharSetSpecDto,
    ChoiceVariantDto,
    CompletenessInfoDto,
    ConstraintDefDto,
    ConstraintDefKindDto,
    ConstraintDto,
    ConstraintEntryDto,
    ConstraintSetDto,
    ConstraintSummaryDto,
    ConstraintTargetMenuDto,
    DiagnosticDto,
    ExprCandidateMenuDto,
    ExpressionDto,
    FillContentDto,
    // New DTOs
    FullProjectionDto,
    HoleCandidateDto,
    LengthSpecDto,
    LiteralDto,
    NodeDetailProjectionDto,
    NodeKindDto,
    OperationErrorDto,
    OutlineNodeDto,
    PropertyTagDto,
    ReferenceCandidateDto,
    ReferenceDto,
    RenderHintKindDto,
    SlotIdDto,
    SlotInfoDto,
    StructureAstDto,
    StructureNodeDto,
    SumBoundDefDto,
    ViolationDetailDto,
    CURRENT_SCHEMA_VERSION,
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

// ── Editor Projection DTOs ──────────────────────────────────────────

/// Convert a [`FullProjection`] to DTO.
#[must_use]
pub fn full_projection_to_dto(fp: &FullProjection) -> FullProjectionDto {
    FullProjectionDto {
        outline: fp.outline.iter().map(outline_node_to_dto).collect(),
        diagnostics: fp.diagnostics.iter().map(diagnostic_to_dto).collect(),
        completeness: completeness_info_to_dto(&fp.completeness),
    }
}

fn outline_node_to_dto(node: &OutlineNode) -> OutlineNodeDto {
    OutlineNodeDto {
        id: node_id_str(node.id),
        label: node.label.clone(),
        kind_label: node.kind_label.clone(),
        depth: node.depth,
        is_hole: node.is_hole,
        child_ids: node_ids_to_strings(&node.child_ids),
    }
}

fn diagnostic_to_dto(diag: &Diagnostic) -> DiagnosticDto {
    DiagnosticDto {
        level: diagnostic_level_str(diag.level),
        message: diag.message.clone(),
        node_id: diag.node_id.map(node_id_str),
        constraint_id: diag.constraint_id.map(constraint_id_str),
    }
}

fn completeness_info_to_dto(info: &CompletenessInfo) -> CompletenessInfoDto {
    CompletenessInfoDto {
        total_holes: info.total_holes,
        is_complete: info.is_complete,
        missing_constraints: info.missing_constraints.clone(),
    }
}

/// Convert a [`NodeDetailProjection`] to DTO.
#[must_use]
pub fn node_detail_to_dto(nd: &NodeDetailProjection) -> NodeDetailProjectionDto {
    NodeDetailProjectionDto {
        slots: nd.slots.iter().map(slot_info_to_dto).collect(),
        related_constraints: nd
            .related_constraints
            .iter()
            .map(constraint_summary_to_dto)
            .collect(),
    }
}

fn slot_info_to_dto(slot: &SlotInfo) -> SlotInfoDto {
    SlotInfoDto {
        kind: slot.kind.as_str().to_owned(),
        current_expr: slot.current_expr.clone(),
        is_editable: slot.is_editable,
    }
}

fn constraint_summary_to_dto(summary: &ConstraintSummary) -> ConstraintSummaryDto {
    ConstraintSummaryDto {
        id: constraint_id_str(summary.id),
        label: summary.label.clone(),
        kind_label: summary.kind_label.clone(),
    }
}

/// Convert hole candidates to DTO.
#[must_use]
pub fn hole_candidates_to_dto(candidates: &[HoleCandidate]) -> Vec<HoleCandidateDto> {
    candidates.iter().map(hole_candidate_to_dto).collect()
}

fn hole_candidate_to_dto(candidate: &HoleCandidate) -> HoleCandidateDto {
    HoleCandidateDto {
        kind: candidate.kind.clone(),
        suggested_names: candidate.suggested_names.clone(),
    }
}

/// Convert expression candidate menu to DTO.
#[must_use]
pub fn expr_candidates_to_dto(menu: &ExprCandidateMenu) -> ExprCandidateMenuDto {
    ExprCandidateMenuDto {
        references: menu
            .references
            .iter()
            .map(reference_candidate_to_dto)
            .collect(),
        literals: menu.literals.clone(),
    }
}

fn reference_candidate_to_dto(candidate: &ReferenceCandidate) -> ReferenceCandidateDto {
    ReferenceCandidateDto {
        node_id: node_id_str(candidate.node_id),
        label: candidate.label.clone(),
    }
}

/// Convert constraint target menu to DTO.
#[must_use]
pub fn constraint_targets_to_dto(menu: &ConstraintTargetMenu) -> ConstraintTargetMenuDto {
    ConstraintTargetMenuDto {
        targets: menu
            .targets
            .iter()
            .map(reference_candidate_to_dto)
            .collect(),
    }
}

// ── Action DTOs ─────────────────────────────────────────────────────

/// Convert an [`Action`] to DTO.
#[must_use]
pub fn action_to_dto(action: &Action) -> ActionDto {
    match action {
        Action::FillHole { target, fill } => ActionDto::FillHole {
            target: node_id_str(*target),
            fill: fill_content_to_dto(fill),
        },
        Action::ReplaceNode {
            target,
            replacement,
        } => ActionDto::ReplaceNode {
            target: node_id_str(*target),
            replacement: fill_content_to_dto(replacement),
        },
        Action::AddConstraint { target, constraint } => ActionDto::AddConstraint {
            target: node_id_str(*target),
            constraint: constraint_def_to_dto(constraint),
        },
        Action::RemoveConstraint { constraint_id } => ActionDto::RemoveConstraint {
            constraint_id: constraint_id_str(*constraint_id),
        },
        Action::IntroduceMultiTestCase {
            count_var_name,
            sum_bound,
        } => ActionDto::IntroduceMultiTestCase {
            count_var_name: count_var_name.clone(),
            sum_bound: sum_bound.as_ref().map(sum_bound_def_to_dto),
        },
        Action::AddSlotElement {
            parent,
            slot_name,
            element,
        } => ActionDto::AddSlotElement {
            parent: node_id_str(*parent),
            slot_name: slot_name.clone(),
            element: fill_content_to_dto(element),
        },
        Action::RemoveSlotElement {
            parent,
            slot_name,
            child,
        } => ActionDto::RemoveSlotElement {
            parent: node_id_str(*parent),
            slot_name: slot_name.clone(),
            child: node_id_str(*child),
        },
        Action::SetExpr { slot, expr } => ActionDto::SetExpr {
            slot: slot_id_to_dto(slot),
            expr: expr_to_dto(expr),
        },
    }
}

fn slot_id_to_dto(slot: &SlotId) -> SlotIdDto {
    SlotIdDto {
        owner: node_id_str(slot.owner),
        kind: slot.kind.as_str().to_owned(),
    }
}

fn fill_content_to_dto(fill: &FillContent) -> FillContentDto {
    match fill {
        FillContent::Scalar { name, typ } => FillContentDto::Scalar {
            name: name.clone(),
            typ: var_type_str(typ),
        },
        FillContent::Array {
            name,
            element_type,
            length,
        } => FillContentDto::Array {
            name: name.clone(),
            element_type: var_type_str(element_type),
            length: length_spec_to_dto(length),
        },
        FillContent::Grid {
            name,
            rows,
            cols,
            cell_type,
        } => FillContentDto::Grid {
            name: name.clone(),
            rows: length_spec_to_dto(rows),
            cols: length_spec_to_dto(cols),
            cell_type: var_type_str(cell_type),
        },
        FillContent::Section { label } => FillContentDto::Section {
            label: label.clone(),
        },
        FillContent::OutputSingleValue { typ } => FillContentDto::OutputSingleValue {
            typ: var_type_str(typ),
        },
        FillContent::OutputYesNo => FillContentDto::OutputYesNo,
    }
}

fn length_spec_to_dto(spec: &LengthSpec) -> LengthSpecDto {
    match spec {
        LengthSpec::Fixed(value) => LengthSpecDto::Fixed { value: *value },
        LengthSpec::RefVar(node_id) => LengthSpecDto::RefVar {
            node_id: node_id_str(*node_id),
        },
        LengthSpec::Expr(value) => LengthSpecDto::Expr {
            value: value.clone(),
        },
    }
}

fn constraint_def_to_dto(def: &ConstraintDef) -> ConstraintDefDto {
    ConstraintDefDto {
        kind: constraint_def_kind_to_dto(&def.kind),
    }
}

fn constraint_def_kind_to_dto(kind: &ConstraintDefKind) -> ConstraintDefKindDto {
    match kind {
        ConstraintDefKind::Range { lower, upper } => ConstraintDefKindDto::Range {
            lower: lower.clone(),
            upper: upper.clone(),
        },
        ConstraintDefKind::TypeDecl { typ } => ConstraintDefKindDto::TypeDecl {
            typ: var_type_str(typ),
        },
        ConstraintDefKind::Relation { op, rhs } => ConstraintDefKindDto::Relation {
            op: relation_op_str(*op),
            rhs: rhs.clone(),
        },
        ConstraintDefKind::Distinct => ConstraintDefKindDto::Distinct,
        ConstraintDefKind::Sorted { order } => ConstraintDefKindDto::Sorted {
            order: sort_order_str(*order),
        },
        ConstraintDefKind::Property { tag } => ConstraintDefKindDto::Property { tag: tag.clone() },
        ConstraintDefKind::SumBound { over_var, upper } => ConstraintDefKindDto::SumBound {
            over_var: over_var.clone(),
            upper: upper.clone(),
        },
        ConstraintDefKind::Guarantee { description } => ConstraintDefKindDto::Guarantee {
            description: description.clone(),
        },
    }
}

fn sum_bound_def_to_dto(def: &SumBoundDef) -> SumBoundDefDto {
    SumBoundDefDto {
        bound_var: def.bound_var.clone(),
        upper: def.upper.clone(),
    }
}

// ── OperationError DTOs ─────────────────────────────────────────────

/// Convert an [`OperationError`] to DTO.
#[must_use]
pub fn operation_error_to_dto(err: &OperationError) -> OperationErrorDto {
    match err {
        OperationError::TypeMismatch {
            expected,
            actual,
            context,
        } => OperationErrorDto::TypeMismatch {
            expected: expected.to_string(),
            actual: actual.clone(),
            context: context.clone(),
        },
        OperationError::NodeNotFound { node } => OperationErrorDto::NodeNotFound {
            node_id: node_id_str(*node),
        },
        OperationError::SlotOccupied {
            node,
            current_occupant,
        } => OperationErrorDto::SlotOccupied {
            node_id: node_id_str(*node),
            current_occupant: current_occupant.clone(),
        },
        OperationError::ConstraintViolation {
            violated_constraints,
        } => OperationErrorDto::ConstraintViolation {
            violations: violated_constraints
                .iter()
                .map(violation_detail_to_dto)
                .collect(),
        },
        OperationError::InvalidOperation { action, reason } => {
            OperationErrorDto::InvalidOperation {
                action: action.clone(),
                reason: reason.clone(),
            }
        }
        OperationError::InvalidFill { reason } => OperationErrorDto::InvalidFill {
            reason: reason.clone(),
        },
        OperationError::DeserializationError { message } => {
            OperationErrorDto::DeserializationError {
                message: message.clone(),
            }
        }
    }
}

fn violation_detail_to_dto(detail: &ViolationDetail) -> ViolationDetailDto {
    ViolationDetailDto {
        constraint_id: constraint_id_str(detail.constraint_id),
        description: detail.description.clone(),
        suggestion: detail.suggestion.clone(),
    }
}

/// Convert an [`ApplyResult`] to DTO.
#[must_use]
pub fn apply_result_to_dto(result: &ApplyResult) -> ApplyResultDto {
    ApplyResultDto {
        created_nodes: node_ids_to_strings(&result.created_nodes),
        removed_nodes: node_ids_to_strings(&result.removed_nodes),
        created_constraints: result
            .created_constraints
            .iter()
            .copied()
            .map(constraint_id_str)
            .collect(),
        affected_constraints: result
            .affected_constraints
            .iter()
            .copied()
            .map(constraint_id_str)
            .collect(),
    }
}

// ── Additional Helper Functions ─────────────────────────────────────

fn diagnostic_level_str(level: DiagnosticLevel) -> String {
    match level {
        DiagnosticLevel::Error => "error".to_owned(),
        DiagnosticLevel::Warning => "warning".to_owned(),
        DiagnosticLevel::Info => "info".to_owned(),
    }
}

fn var_type_str(typ: &VarType) -> String {
    match typ {
        VarType::Int => "Int".to_owned(),
        VarType::Str => "Str".to_owned(),
        VarType::Char => "Char".to_owned(),
    }
}
