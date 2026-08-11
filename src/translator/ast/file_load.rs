use tree_sitter::Node;

use crate::translator::ast::AstBuilder;

impl AstBuilder {
    pub fn get_file_load(&self, node: &Node) -> String {
        self.expect_node_kind(node, "file_load");
        self.get_string(&node.child_by_field_name("filename").unwrap())
    }

    pub fn get_comment(&self, node: &Node) -> String {
        self.expect_node_kind(node, "comment");
        self.get_string(node)
    }
}
