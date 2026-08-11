use tree_sitter::Node;

use crate::translator::ast::Range;

use super::{Sequence, Signature, AstBuilder, VarDefinition, Assignment};

#[derive(Debug)]
pub enum Statement {
    Event(Sequence),
    VarDefinition(VarDefinition),
    Assignment(Assignment),
    Definition(Definition),
}

#[derive(Debug)]
pub struct Definition {
    pub signature: Signature,
    pub body: Vec<(Statement,Range)>,
}

impl Definition {
    pub fn find_variable_definition(&self, name: &str) -> Range {
        for (stmt, _) in &self.body {
            let Statement::VarDefinition(d) = stmt else { continue; };
            if &d.name.0 == name {
                return d.name.1.clone();
            }
        }
        panic!("could not find variable definition for `{name}`");
    }
}

impl AstBuilder {
    pub fn get_definition(&self, node: &Node) -> Definition {
        self.expect_node_kind(node, "definition");
        let signature = self.get_signature(&node.child_by_field_name("signature").unwrap());
        let body = self.get_body(&node.child_by_field_name("body").unwrap());
        Definition { signature, body }
    }

    fn get_body(&self, node: &Node) -> Vec<(Statement, Range)> {
        self.expect_node_kind(node, "definition_body");
        let mut stmts = vec![];
        for s in node.children(&mut node.walk()) {
            match s.kind() {
                "comment" | "{" | "}" | ";" => {}
                "sequence" => stmts.push((Statement::Event(self.get_sequence(&s)), Range::from(&s))),
                "definition" => stmts.push((Statement::Definition(self.get_definition(&s)), Range::from(&s))),
                "var_definition" => stmts.push((Statement::VarDefinition(self.get_var_definition(&s)), Range::from(&s))),
                "assignment" => stmts.push((Statement::Assignment(self.get_var_assignment(&s)), Range::from(&s))),
                x => panic!("error: unexpected node kind for definition body: `{x}")
            }
        }
        stmts
    }
}
