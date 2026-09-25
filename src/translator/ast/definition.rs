use tree_sitter::Node;

use crate::translator::ast::{Event, Range};

use super::{Signature, AstBuilder};

#[derive(Debug)]
pub enum Statement {
    Event(Event),
    Definition(Definition),
}

impl From<&Statement> for Range {
    fn from(value: &Statement) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct Definition {
    pub signature: Signature,
    pub body: Vec<Statement>,
    // TODO: keep range between curly braces
}

impl Definition {
    pub fn find_variable_definition(&self, name: &str) -> Range {
        todo!();
        // for (w, r) in &self.signature {
        //     let param = match w {
        //         super::signature::Word::Variable(name) => name,
        //         super::signature::Word::Iterator(it) => &it.0,
        //         _ => continue,
        //     };
        //     if param == name {
        //         return r.clone();
        //     }
        // }
        // for (stmt, _) in &self.body {
        //     let Statement::VarDefinition(d) = stmt else { continue; };
        //     if &d.name.0 == name {
        //         return d.name.1.clone();
        //     }
        // }
        // panic!("could not find variable definition for `{name}`");
    }

    pub fn find_nth_method(&self, i: usize) -> &Definition {
        let Statement::Definition(method) = &self.body
            .iter()
            .filter(|s| matches!(s, Statement::Definition(_)))
            .skip(i)
            .next()
            .expect(&format!("error: definition `{:?}` does not have {i} methods", self.signature.iter().map(|(w,_)| w).collect::<Vec<_>>()))
            else {
                panic!("error: failed to retrieve a method")
            };
        method
    }
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
                "definition" => stmts.push(Statement::Definition(self.get_definition(&s))),
                "event" => stmts.push(Statement::Event(self.get_event(&s))),
                x => panic!("error: unexpected node kind for definition body: `{x}")
            }
        }
        stmts
    }
}
