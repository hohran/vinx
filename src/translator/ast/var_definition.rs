use tree_sitter::Node;

use super::{Sequence, AstBuilder};

#[derive(Debug)]
pub struct VarDefinition {
    pub name: String,
    pub value: Sequence,
}

#[derive(Debug)]
pub struct Assignment {
    pub name: String,
    pub value: Sequence,
}

impl AstBuilder {
    pub fn get_var_definition(&self, node: &Node) -> VarDefinition {
        self.expect_node_kind(node, "var_definition");
        let name = self.get_variable(&node.child_by_field_name("lhs").unwrap());
        let value = self.get_sequence(&node.child_by_field_name("rhs").unwrap());
        VarDefinition { name, value }
    }

    pub fn get_var_assignment(&self, node: &Node) -> Assignment {
        self.expect_node_kind(node, "assignment");
        let name = self.get_variable(&node.child_by_field_name("lhs").unwrap());
        let value = self.get_sequence(&node.child_by_field_name("rhs").unwrap());
        Assignment { name, value }
    }
}
