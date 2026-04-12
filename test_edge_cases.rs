use cp_ast_core::constraint::{ConstraintSet, Constraint};
use cp_ast_core::structure::{NodeId, Reference, Expression, ExpectedType};

fn main() {
    let mut set = ConstraintSet::new();
    
    // Test: double remove
    let id = set.add(None, Constraint::Guarantee {
        description: "test".to_owned(),
        predicate: None,
    });
    
    println!("First remove: {:?}", set.remove(id).is_some());
    println!("Second remove: {:?}", set.remove(id).is_some());
    
    // Test: for_node with non-existent node
    let non_existent = NodeId::from_raw(999);
    let constraints = set.for_node(non_existent);
    println!("Non-existent node constraints: {:?}", constraints.len());
}
