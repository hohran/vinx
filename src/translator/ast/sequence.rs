use tree_sitter::Node;

use crate::translator::ast::AstBuilder;

use super::Value;

#[derive(Debug)]
pub enum Word {
    Keyword(String),
    Value(Value)
}

pub type Sequence = Vec<Word>;

impl AstBuilder {
    pub fn get_sequence(&self, node: &Node) -> Sequence {
        self.expect_node_kind(node, "sequence");
        let mut seq = vec![];
        for word in node.children(&mut node.walk()) {
            match word.kind() {
                "comment" => {}
                "keyword" => seq.push(Word::Keyword(self.get_keyword(&word))),
                "value" => seq.push(Word::Value(self.get_value(&word))),
                x => panic!("error: unexpected node kind in sequence {node:?}: {x}"),
            }
        }
        seq
    }

    pub fn get_keyword(&self, node: &Node) -> String {
        self.expect_node_kind(node, "keyword");
        self.text(node).to_string()
    }
}
