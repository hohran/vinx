use tree_sitter::Node;

use crate::translator::ast::Range;

use super::{Sequence, AstBuilder, Type};

#[derive(Debug, Clone)]
pub struct VarDefinition {
    pub name: (String, Range),
    pub typ: Option<(Type, Range)>,
    pub value: Option<(Sequence, Range)>,
}

#[derive(Debug)]
pub struct Assignment {
    pub name: (String, Range),
    pub value: (Sequence, Range),
}

impl AstBuilder {
    pub fn get_var_definition(&self, node: &Node) -> VarDefinition {
        self.expect_node_kind(node, "var_definition");
        let name_node = node.child_by_field_name("name").unwrap();
        let name = (self.get_variable(&name_node), Range::from(&name_node));
        let typ = node.child_by_field_name("type").map(|type_node|
            (self.get_type(&type_node), Range::from(&type_node)));
        let value = node.child_by_field_name("val").map(|value_node|
            (self.get_sequence(&value_node), Range::from(&value_node)));
        VarDefinition { name, value, typ }
    }

    pub fn get_var_assignment(&self, node: &Node) -> Assignment {
        self.expect_node_kind(node, "assignment");
        let name_node = node.child_by_field_name("lhs").unwrap();
        let name = (self.get_variable(&name_node), Range::from(&name_node));
        let value_node = node.child_by_field_name("rhs").unwrap();
        let value = (self.get_sequence(&value_node), Range::from(&value_node));
        Assignment { name, value }
    }
}
