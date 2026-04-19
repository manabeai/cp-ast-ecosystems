use cp_ast_core::operation::*;
use cp_ast_core::structure::*;

/// Helper: create an engine, add a scalar to root sequence, return (engine, `scalar_id`).
fn engine_with_scalar(name: &str) -> (AstEngine, NodeId) {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();
    let result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: name.to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();
    let id = *result.created_nodes.last().unwrap();
    (engine, id)
}

// ── AddSibling: wraps scalar in tuple ───────────────────────────────────

#[test]
fn add_sibling_wraps_scalar_in_tuple() {
    let (mut engine, h_id) = engine_with_scalar("H");

    // AddSibling: add W next to H
    let result = engine
        .apply(&Action::AddSibling {
            target: h_id,
            element: FillContent::Scalar {
                name: "W".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    // A tuple node should have been created
    assert!(
        !result.created_nodes.is_empty(),
        "AddSibling should create nodes"
    );

    // Root's children should now contain a Tuple (not H directly)
    let root = engine.structure.root();
    let root_node = engine.structure.get(root).unwrap();
    if let NodeKind::Sequence { children } = root_node.kind() {
        assert_eq!(children.len(), 1, "Root should have 1 child (the Tuple)");
        let tuple_id = children[0];
        let tuple_node = engine.structure.get(tuple_id).unwrap();
        if let NodeKind::Tuple { elements } = tuple_node.kind() {
            assert_eq!(elements.len(), 2, "Tuple should have H and W");
            // H should still be elements[0]
            assert_eq!(elements[0], h_id);
            // W should be elements[1]
            let w_node = engine.structure.get(elements[1]).unwrap();
            if let NodeKind::Scalar { name } = w_node.kind() {
                assert_eq!(name.as_str(), "W");
            } else {
                panic!("Expected Scalar W, got {:?}", w_node.kind());
            }
        } else {
            panic!("Expected Tuple, got {:?}", tuple_node.kind());
        }
    } else {
        panic!("Expected Sequence root, got {:?}", root_node.kind());
    }
}

// ── AddSibling: appends to existing tuple ───────────────────────────────

#[test]
fn add_sibling_to_existing_tuple_appends() {
    let (mut engine, h_id) = engine_with_scalar("H");

    // First AddSibling: wraps H in Tuple[H, W]
    let r1 = engine
        .apply(&Action::AddSibling {
            target: h_id,
            element: FillContent::Scalar {
                name: "W".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    // Find W's id
    let w_id = r1
        .created_nodes
        .iter()
        .copied()
        .find(|&id| {
            matches!(
                engine.structure.get(id).map(cp_ast_core::structure::StructureNode::kind),
                Some(NodeKind::Scalar { name }) if name.as_str() == "W"
            )
        })
        .expect("W should have been created");

    // Second AddSibling: add K next to W (W is inside the Tuple)
    engine
        .apply(&Action::AddSibling {
            target: w_id,
            element: FillContent::Scalar {
                name: "K".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    // Now Tuple should have [H, W, K]
    let root = engine.structure.root();
    let root_node = engine.structure.get(root).unwrap();
    if let NodeKind::Sequence { children } = root_node.kind() {
        assert_eq!(children.len(), 1, "Root should still have 1 child");
        let tuple_node = engine.structure.get(children[0]).unwrap();
        if let NodeKind::Tuple { elements } = tuple_node.kind() {
            assert_eq!(elements.len(), 3, "Tuple should have H, W, K");
            // Verify names
            let names: Vec<&str> = elements
                .iter()
                .map(|&id| {
                    if let NodeKind::Scalar { name } = engine.structure.get(id).unwrap().kind() {
                        name.as_str()
                    } else {
                        panic!("Expected Scalar");
                    }
                })
                .collect();
            assert_eq!(names, vec!["H", "W", "K"]);
        } else {
            panic!("Expected Tuple, got {:?}", tuple_node.kind());
        }
    } else {
        panic!("Expected Sequence root");
    }
}

// ── AddChoiceVariant: adds variant to choice ────────────────────────────

#[test]
fn add_choice_variant_adds_to_choice() {
    let mut engine = AstEngine::new();
    let root = engine.structure.root();

    // Create a QueryList to get a Choice inside Repeat
    let q_result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::Scalar {
                name: "Q".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();
    let q_id = *q_result.created_nodes.last().unwrap();

    let query_result = engine
        .apply(&Action::AddSlotElement {
            parent: root,
            slot_name: "children".to_owned(),
            element: FillContent::QueryList {
                query_count: LengthSpec::RefVar(q_id),
            },
        })
        .unwrap();

    // Find the Choice node from the created nodes
    let choice_id = query_result
        .created_nodes
        .iter()
        .copied()
        .find(|&id| {
            matches!(
                engine
                    .structure
                    .get(id)
                    .map(cp_ast_core::structure::StructureNode::kind),
                Some(NodeKind::Choice { .. })
            )
        })
        .expect("QueryList should create a Choice node");

    // Verify choice starts empty
    if let NodeKind::Choice { variants, .. } = engine.structure.get(choice_id).unwrap().kind() {
        assert!(variants.is_empty(), "Choice should start with no variants");
    }

    // AddChoiceVariant: add variant with tag=IntLit(1), body element=Scalar "a"
    let result = engine
        .apply(&Action::AddChoiceVariant {
            choice: choice_id,
            tag_value: Literal::IntLit(1),
            first_element: FillContent::Scalar {
                name: "a".to_owned(),
                typ: VarType::Int,
            },
        })
        .unwrap();

    assert!(
        !result.created_nodes.is_empty(),
        "AddChoiceVariant should create nodes"
    );

    // Verify variant was added
    let choice_node = engine.structure.get(choice_id).unwrap();
    if let NodeKind::Choice { variants, .. } = choice_node.kind() {
        assert_eq!(variants.len(), 1, "Choice should have 1 variant");
        let (tag, body) = &variants[0];
        assert_eq!(*tag, Literal::IntLit(1));
        assert_eq!(body.len(), 1, "Variant body should have 1 element");
        // Verify the body element is a Scalar "a"
        let elem_node = engine.structure.get(body[0]).unwrap();
        if let NodeKind::Scalar { name } = elem_node.kind() {
            assert_eq!(name.as_str(), "a");
        } else {
            panic!("Expected Scalar 'a', got {:?}", elem_node.kind());
        }
    } else {
        panic!("Expected Choice, got {:?}", choice_node.kind());
    }
}
