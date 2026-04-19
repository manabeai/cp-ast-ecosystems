//! Bidirectional conversion between `Action` ↔ `ActionDto`.

use cp_ast_core::constraint::{ConstraintId, RelationOp, SortOrder};
use cp_ast_core::operation::action::Action;
use cp_ast_core::operation::types::{
    ConstraintDef, ConstraintDefKind, FillContent, LengthSpec, SumBoundDef, VarType,
};
use cp_ast_core::structure::{Literal, NodeId};

use crate::dto::{
    ActionDto, ConstraintDefDto, FillContentDto, LengthSpecDto, LiteralDto, SumBoundDefDto,
};
use crate::error::ConversionError;

// ── public API ──────────────────────────────────────────────────────

/// Serialize an [`Action`] to a JSON string.
///
/// # Errors
/// Returns `ConversionError::Json` if JSON serialization fails.
pub fn serialize_action(action: &Action) -> Result<String, ConversionError> {
    let dto = action_to_dto(action);
    serde_json::to_string(&dto).map_err(ConversionError::from)
}

/// Deserialize an [`Action`] from a JSON string.
///
/// # Errors
/// Returns `ConversionError` if JSON is invalid or enum variants are unknown.
pub fn deserialize_action(json: &str) -> Result<Action, ConversionError> {
    let dto: ActionDto = serde_json::from_str(json)?;
    dto_to_action(&dto)
}

// ── Action → DTO ────────────────────────────────────────────────────

/// Convert an [`Action`] to its DTO representation.
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
            constraint_id: constraint_id.value().to_string(),
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
        Action::AddSibling { target, element } => ActionDto::AddSibling {
            target: node_id_str(*target),
            element: fill_content_to_dto(element),
        },
        Action::AddChoiceVariant {
            choice,
            tag_value,
            first_element,
        } => ActionDto::AddChoiceVariant {
            choice: node_id_str(*choice),
            tag_value: literal_to_dto(tag_value),
            first_element: fill_content_to_dto(first_element),
        },
    }
}

// ── DTO → Action ────────────────────────────────────────────────────

/// Convert an [`ActionDto`] back to an [`Action`].
///
/// # Errors
/// Returns `ConversionError` if IDs are malformed or enum variants unknown.
pub fn dto_to_action(dto: &ActionDto) -> Result<Action, ConversionError> {
    match dto {
        ActionDto::FillHole { target, fill } => Ok(Action::FillHole {
            target: parse_node_id(target)?,
            fill: dto_to_fill_content(fill)?,
        }),
        ActionDto::ReplaceNode {
            target,
            replacement,
        } => Ok(Action::ReplaceNode {
            target: parse_node_id(target)?,
            replacement: dto_to_fill_content(replacement)?,
        }),
        ActionDto::AddConstraint { target, constraint } => Ok(Action::AddConstraint {
            target: parse_node_id(target)?,
            constraint: dto_to_constraint_def(constraint)?,
        }),
        ActionDto::RemoveConstraint { constraint_id } => Ok(Action::RemoveConstraint {
            constraint_id: parse_constraint_id(constraint_id)?,
        }),
        ActionDto::IntroduceMultiTestCase {
            count_var_name,
            sum_bound,
        } => Ok(Action::IntroduceMultiTestCase {
            count_var_name: count_var_name.clone(),
            sum_bound: sum_bound.as_ref().map(dto_to_sum_bound_def),
        }),
        ActionDto::AddSlotElement {
            parent,
            slot_name,
            element,
        } => Ok(Action::AddSlotElement {
            parent: parse_node_id(parent)?,
            slot_name: slot_name.clone(),
            element: dto_to_fill_content(element)?,
        }),
        ActionDto::RemoveSlotElement {
            parent,
            slot_name,
            child,
        } => Ok(Action::RemoveSlotElement {
            parent: parse_node_id(parent)?,
            slot_name: slot_name.clone(),
            child: parse_node_id(child)?,
        }),
        ActionDto::AddSibling { target, element } => Ok(Action::AddSibling {
            target: parse_node_id(target)?,
            element: dto_to_fill_content(element)?,
        }),
        ActionDto::AddChoiceVariant {
            choice,
            tag_value,
            first_element,
        } => Ok(Action::AddChoiceVariant {
            choice: parse_node_id(choice)?,
            tag_value: dto_to_literal(tag_value),
            first_element: dto_to_fill_content(first_element)?,
        }),
    }
}

// ── FillContent helpers ─────────────────────────────────────────────

fn fill_content_to_dto(fc: &FillContent) -> FillContentDto {
    match fc {
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
        FillContent::EdgeList { edge_count } => FillContentDto::EdgeList {
            edge_count: length_spec_to_dto(edge_count),
        },
        FillContent::WeightedEdgeList {
            edge_count,
            weight_name,
            weight_type,
        } => FillContentDto::WeightedEdgeList {
            edge_count: length_spec_to_dto(edge_count),
            weight_name: weight_name.clone(),
            weight_type: var_type_str(weight_type),
        },
        FillContent::QueryList { query_count } => FillContentDto::QueryList {
            query_count: length_spec_to_dto(query_count),
        },
        FillContent::MultiTestCaseTemplate { count } => FillContentDto::MultiTestCaseTemplate {
            count: length_spec_to_dto(count),
        },
        FillContent::GridTemplate {
            name,
            rows,
            cols,
            cell_type,
        } => FillContentDto::GridTemplate {
            name: name.clone(),
            rows: length_spec_to_dto(rows),
            cols: length_spec_to_dto(cols),
            cell_type: var_type_str(cell_type),
        },
    }
}

fn dto_to_fill_content(dto: &FillContentDto) -> Result<FillContent, ConversionError> {
    match dto {
        FillContentDto::Scalar { name, typ } => Ok(FillContent::Scalar {
            name: name.clone(),
            typ: str_to_var_type(typ)?,
        }),
        FillContentDto::Array {
            name,
            element_type,
            length,
        } => Ok(FillContent::Array {
            name: name.clone(),
            element_type: str_to_var_type(element_type)?,
            length: dto_to_length_spec(length)?,
        }),
        FillContentDto::Grid {
            name,
            rows,
            cols,
            cell_type,
        } => Ok(FillContent::Grid {
            name: name.clone(),
            rows: dto_to_length_spec(rows)?,
            cols: dto_to_length_spec(cols)?,
            cell_type: str_to_var_type(cell_type)?,
        }),
        FillContentDto::Section { label } => Ok(FillContent::Section {
            label: label.clone(),
        }),
        FillContentDto::OutputSingleValue { typ } => Ok(FillContent::OutputSingleValue {
            typ: str_to_var_type(typ)?,
        }),
        FillContentDto::OutputYesNo => Ok(FillContent::OutputYesNo),
        FillContentDto::EdgeList { edge_count } => Ok(FillContent::EdgeList {
            edge_count: dto_to_length_spec(edge_count)?,
        }),
        FillContentDto::WeightedEdgeList {
            edge_count,
            weight_name,
            weight_type,
        } => Ok(FillContent::WeightedEdgeList {
            edge_count: dto_to_length_spec(edge_count)?,
            weight_name: weight_name.clone(),
            weight_type: str_to_var_type(weight_type)?,
        }),
        FillContentDto::QueryList { query_count } => Ok(FillContent::QueryList {
            query_count: dto_to_length_spec(query_count)?,
        }),
        FillContentDto::MultiTestCaseTemplate { count } => Ok(FillContent::MultiTestCaseTemplate {
            count: dto_to_length_spec(count)?,
        }),
        FillContentDto::GridTemplate {
            name,
            rows,
            cols,
            cell_type,
        } => Ok(FillContent::GridTemplate {
            name: name.clone(),
            rows: dto_to_length_spec(rows)?,
            cols: dto_to_length_spec(cols)?,
            cell_type: str_to_var_type(cell_type)?,
        }),
    }
}

// ── LengthSpec helpers ──────────────────────────────────────────────

fn length_spec_to_dto(ls: &LengthSpec) -> LengthSpecDto {
    match ls {
        LengthSpec::Fixed(n) => LengthSpecDto::Fixed { value: *n },
        LengthSpec::RefVar(id) => LengthSpecDto::RefVar {
            node_id: node_id_str(*id),
        },
        LengthSpec::Expr(s) => LengthSpecDto::Expr { expr: s.clone() },
    }
}

fn dto_to_length_spec(dto: &LengthSpecDto) -> Result<LengthSpec, ConversionError> {
    match dto {
        LengthSpecDto::Fixed { value } => Ok(LengthSpec::Fixed(*value)),
        LengthSpecDto::RefVar { node_id } => Ok(LengthSpec::RefVar(parse_node_id(node_id)?)),
        LengthSpecDto::Expr { expr } => Ok(LengthSpec::Expr(expr.clone())),
    }
}

// ── ConstraintDef helpers ───────────────────────────────────────────

fn constraint_def_to_dto(cd: &ConstraintDef) -> ConstraintDefDto {
    match &cd.kind {
        ConstraintDefKind::Range { lower, upper } => ConstraintDefDto::Range {
            lower: lower.clone(),
            upper: upper.clone(),
        },
        ConstraintDefKind::TypeDecl { typ } => ConstraintDefDto::TypeDecl {
            typ: var_type_str(typ),
        },
        ConstraintDefKind::Relation { op, rhs } => ConstraintDefDto::Relation {
            op: relation_op_str(*op),
            rhs: rhs.clone(),
        },
        ConstraintDefKind::Distinct => ConstraintDefDto::Distinct,
        ConstraintDefKind::Sorted { order } => ConstraintDefDto::Sorted {
            order: sort_order_str(*order),
        },
        ConstraintDefKind::Property { tag } => ConstraintDefDto::Property { tag: tag.clone() },
        ConstraintDefKind::SumBound { over_var, upper } => ConstraintDefDto::SumBound {
            over_var: over_var.clone(),
            upper: upper.clone(),
        },
        ConstraintDefKind::Guarantee { description } => ConstraintDefDto::Guarantee {
            description: description.clone(),
        },
    }
}

fn dto_to_constraint_def(dto: &ConstraintDefDto) -> Result<ConstraintDef, ConversionError> {
    let kind = match dto {
        ConstraintDefDto::Range { lower, upper } => ConstraintDefKind::Range {
            lower: lower.clone(),
            upper: upper.clone(),
        },
        ConstraintDefDto::TypeDecl { typ } => ConstraintDefKind::TypeDecl {
            typ: str_to_var_type(typ)?,
        },
        ConstraintDefDto::Relation { op, rhs } => ConstraintDefKind::Relation {
            op: str_to_relation_op(op)?,
            rhs: rhs.clone(),
        },
        ConstraintDefDto::Distinct => ConstraintDefKind::Distinct,
        ConstraintDefDto::Sorted { order } => ConstraintDefKind::Sorted {
            order: str_to_sort_order(order)?,
        },
        ConstraintDefDto::Property { tag } => ConstraintDefKind::Property { tag: tag.clone() },
        ConstraintDefDto::SumBound { over_var, upper } => ConstraintDefKind::SumBound {
            over_var: over_var.clone(),
            upper: upper.clone(),
        },
        ConstraintDefDto::Guarantee { description } => ConstraintDefKind::Guarantee {
            description: description.clone(),
        },
    };
    Ok(ConstraintDef { kind })
}

// ── SumBoundDef helpers ─────────────────────────────────────────────

fn sum_bound_def_to_dto(sb: &SumBoundDef) -> SumBoundDefDto {
    SumBoundDefDto {
        bound_var: sb.bound_var.clone(),
        upper: sb.upper.clone(),
    }
}

fn dto_to_sum_bound_def(dto: &SumBoundDefDto) -> SumBoundDef {
    SumBoundDef {
        bound_var: dto.bound_var.clone(),
        upper: dto.upper.clone(),
    }
}

// ── Literal helpers ─────────────────────────────────────────────────

fn literal_to_dto(l: &Literal) -> LiteralDto {
    match l {
        Literal::IntLit(v) => LiteralDto::IntLit { value: *v },
        Literal::StrLit(s) => LiteralDto::StrLit { value: s.clone() },
    }
}

fn dto_to_literal(dto: &LiteralDto) -> Literal {
    match dto {
        LiteralDto::IntLit { value } => Literal::IntLit(*value),
        LiteralDto::StrLit { value } => Literal::StrLit(value.clone()),
    }
}

// ── small-enum string helpers ───────────────────────────────────────

fn node_id_str(id: NodeId) -> String {
    id.value().to_string()
}

fn parse_node_id(s: &str) -> Result<NodeId, ConversionError> {
    s.parse::<u64>()
        .map(NodeId::from_raw)
        .map_err(|_| ConversionError::InvalidId(s.to_owned()))
}

fn parse_constraint_id(s: &str) -> Result<ConstraintId, ConversionError> {
    s.parse::<u64>()
        .map(ConstraintId::from_raw)
        .map_err(|_| ConversionError::InvalidId(s.to_owned()))
}

fn var_type_str(vt: &VarType) -> String {
    match vt {
        VarType::Int => "Int".to_owned(),
        VarType::Str => "Str".to_owned(),
        VarType::Char => "Char".to_owned(),
    }
}

fn str_to_var_type(s: &str) -> Result<VarType, ConversionError> {
    match s {
        "Int" => Ok(VarType::Int),
        "Str" => Ok(VarType::Str),
        "Char" => Ok(VarType::Char),
        _ => Err(ConversionError::UnknownVariant {
            type_name: "VarType",
            value: s.to_owned(),
        }),
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

fn sort_order_str(o: SortOrder) -> String {
    match o {
        SortOrder::Ascending => "Ascending".to_owned(),
        SortOrder::NonDecreasing => "NonDecreasing".to_owned(),
        SortOrder::Descending => "Descending".to_owned(),
        SortOrder::NonIncreasing => "NonIncreasing".to_owned(),
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
