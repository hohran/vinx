use tree_sitter::Node;

use super::{Sequence, AstBuilder};

pub struct VarDefinition {
    pub name: String,
    pub value: Sequence,
}

impl AstBuilder {
    pub fn get_var_definition(&self, node: &Node) -> VarDefinition {
        let name = self.get_variable(&node.child_by_field_name("lhs").unwrap());
        let value = self.get_sequence(&node.child_by_field_name("rhs").unwrap());
        VarDefinition { name, value }
    }
}
