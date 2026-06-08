use tree_sitter::Node;

use super::{Sequence, Signature, AstBuilder, VarDefinition};

pub enum Statement {
    Event(Sequence),
    VarDefinition(VarDefinition),
    Definition(Definition),
}

pub struct Definition {
    pub signature: Signature,
    pub body: Vec<Statement>
}

impl AstBuilder {
    pub fn get_definition(&self, node: &Node) -> Definition {
        self.expect_node_kind(node, "definition");
        let signature = self.get_signature(&node.child_by_field_name("signature").unwrap());
        let body = self.get_body(&node.child_by_field_name("body").unwrap());
        Definition { signature, body }
    }

    fn get_body(&self, node: &Node) -> Vec<Statement> {
        self.expect_node_kind(node, "definition_body");
        let mut stmts = vec![];
        for s in node.children(&mut node.walk()) {
            match s.kind() {
                "comment" | "{" | "}" | ";" => {}
                "sequence" => stmts.push(Statement::Event(self.get_sequence(&s))),
                "definition" => stmts.push(Statement::Definition(self.get_definition(&s))),
                "var_definition" => stmts.push(Statement::VarDefinition(self.get_var_definition(&s))),
                x => panic!("error: unexpected node kind for definition body: `{x}")
            }
        }
        stmts
    }
}
