use core::panic;

use tree_sitter::Node;

use crate::translator::ast::AstBuilder;
use super::Range;

pub type Iterator = (String, bool); // variable name, is_main

#[derive(Debug)]
pub enum Word {
    Keyword(String),
    Variable(String),
    Iterator(Iterator),
}

// NOTE: range of signature can be derived from its words
pub type Signature = Vec<(Word, Range)>;

impl AstBuilder {
    pub fn get_signature(&self, node: &Node) -> Signature {
        self.expect_node_kind(node, "signature");
        let mut sign = vec![];
        for word in node.children(&mut node.walk()) {
            match word.kind() {
                "comment" => {}
                "keyword" => sign.push((Word::Keyword(self.get_keyword(&word)), Range::from(&word))),
                "variable" => sign.push((Word::Variable(self.get_variable(&word)), Range::from(&word))),
                "iterator" => sign.push((Word::Iterator(self.get_iterator(&word)), Range::from(&word))),
                "ERROR" => { // TODO: what to do with errors?
                    if self.text(&word) == "=" {
                        panic!("{}:{}:{}: tried to do assignment in var decl.", self.filename, word.range().start_point.row, word.range().start_point.column);
                    }
                    panic!("i dont know the reason for this error: {word:?}");
                }
                x => panic!("error: unexpected node kind in signature {word:?}: {x}"),
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
