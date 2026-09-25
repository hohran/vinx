use crate::translator::ast::Event;

use super::{Action, Definition, AstBuilder};

pub enum AstNode {
    Action(Action),
    Definition(Definition), // TODO: test
    Event(Event), // TODO: test
}

pub struct Ast {
    pub nodes: Vec<AstNode>,
}

impl Ast {
    pub fn parse(filepath: &str) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_vinx::LANGUAGE.into()).expect("error: could not load vinx grammar");
        let contents = std::fs::read_to_string(filepath).expect("error reading input file");
        Self::parse_from_contents(filepath, contents)
    }

    pub fn parse_from_contents(filepath: &str, contents: String) -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_vinx::LANGUAGE.into()).expect("error: could not load vinx grammar");
        let tree = parser.parse(&contents, None).unwrap();
        let root_node = tree.root_node();
        let builder = AstBuilder::new(filepath, contents.to_string());
        Self::parse_from_root_node(builder, root_node)
    }

    pub fn parse_from_root_node(builder: AstBuilder, root_node: tree_sitter::Node) -> Self {
        let mut nodes = vec![];
        for node in root_node.children(&mut root_node.walk()) {
            let ast_node = match node.kind() {
                "comment" => continue,
                "action" => AstNode::Action(builder.get_action(&node)),
                "definition" => AstNode::Definition(builder.get_definition(&node)),
                "event" => AstNode::Event(builder.get_event(&node)),
                x => panic!("error: unexpected top-level node: `{x}")
            };
            nodes.push(ast_node);
        }
        Ast { nodes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::macros::*;

    #[test]
    fn test_ast() {
        let ast = ast!("
            $positions := [(0,0),(1,1),(2,2)]; // event
            print $positions one by one;       // event
            $positions = [(1,1)];              // event
            every frame do something;        // action
            every frame { do something; }    // action
            new function := { do something; }   // definition
            process [$x] := { process $x; }     // definition
        ");
        assert_eq!(ast.nodes.len(), 7);
        let event_count = ast.nodes.iter().filter(|n| matches!(n, AstNode::Event(_))).count();
        assert_eq!(event_count, 3);
        let action_count = ast.nodes.iter().filter(|n| matches!(n, AstNode::Action(_))).count();
        assert_eq!(action_count, 2);
        let definition_count = ast.nodes.iter().filter(|n| matches!(n, AstNode::Definition(_))).count();
        assert_eq!(definition_count, 2);
        assert_eq!(ast!("").nodes.len(), 0); // empty file
    }
}
