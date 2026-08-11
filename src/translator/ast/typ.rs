use tree_sitter::Node;

use super::AstBuilder;

#[derive(Debug, Clone)]
pub struct Type {
    pub value: String, // type value is Int, String, Image, etc.
    pub depth: usize,
}

impl AstBuilder {
    fn get_type_val(&self, node: &Node) -> String {
        self.expect_node_kind(node, "type_val");
        self.text(node).to_string()
    }

    fn get_nested_type(&self, node: &Node, depth: usize) -> Type {
        self.expect_node_kind(node, "type_nest");
        for n in node.children(&mut node.walk()) {
            if n.kind() == "type" {
                let child = n.child(0).unwrap();
                if child.kind() == "type_val" {
                    let value = self.get_type_val(&child);
                    return Type { value, depth }
                } else {
                    return self.get_nested_type(&child.child(0).unwrap(), depth+1)
                }
            }
        }
        panic!("error: nested type did not contain a node with type `type`");
    }

    pub fn get_type(&self, node: &Node) -> Type {
        self.expect_node_kind(node, "type");
        let child = node.child(0).unwrap();
        if child.kind() == "type_nest" {
            self.get_nested_type(&child, 0)
        } else {
            let value = self.get_type_val(&child);
            return Type { value, depth: 0 }
        }
    }
}
