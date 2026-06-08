use tree_sitter::Node;

use crate::translator::ast::AstBuilder;

pub type Iterator = (String, bool); // variable name, is_main

pub enum Word {
    Keyword(String),
    Variable(String),
    Iterator(Iterator),
}

pub type Signature = Vec<Word>;

impl AstBuilder {
    pub fn get_signature(&self, node: &Node) -> Signature {
        self.expect_node_kind(node, "signature");
        let mut sign = vec![];
        for word in node.children(&mut node.walk()) {
            match word.kind() {
                "comment" => {}
                "keyword" => sign.push(Word::Keyword(self.get_keyword(&word))),
                "variable" => sign.push(Word::Variable(self.get_variable(&word))),
                "iterator" => sign.push(Word::Iterator(self.get_iterator(&word))),
                x => panic!("error: unexpected node kind in signature {node:?}: {x}"),
            }
        }
        sign
    }

    pub fn get_iterator(&self, node: &Node) -> Iterator {
        self.expect_node_kind(node, "iterator");
        let var = self.get_variable(&node.child_by_field_name("variable").unwrap());
        let is_main = node.child_by_field_name("main").is_some();
        (var, is_main)
    }
}
