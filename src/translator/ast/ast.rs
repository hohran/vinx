use crate::translator::ast::{Range, Sequence};

use super::{Action, VarDefinition, Definition, AstBuilder};

pub enum AstNode {
    Action(Action),
    VarDefinition(VarDefinition),
    Definition(Definition),
    Sequence(Sequence),
    // FileLoad(String),
    // Comment(String),
}

pub struct Ast {
    pub nodes: Vec<(AstNode, Range)>,
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
        let mut nodes = vec![];
        for node in root_node.children(&mut root_node.walk()) {
            let ast_node = match node.kind() {
                "comment" | ";" => { continue; }
                "action" => AstNode::Action(builder.get_action(&node)),
                "var_definition" => AstNode::VarDefinition(builder.get_var_definition(&node)),
                "definition" => AstNode::Definition(builder.get_definition(&node)),
                "sequence" => AstNode::Sequence(builder.get_sequence(&node)),
                x => panic!("error: unexpected top-level node: `{x}")
            };
            nodes.push((ast_node, Range::from(&node)));
        }
        Ast { nodes }
    }

    pub fn parse_from_root_node(builder: AstBuilder, root_node: tree_sitter::Node) -> Self {
        let mut nodes = vec![];
        for node in root_node.children(&mut root_node.walk()) {
            let ast_node = match node.kind() {
                "comment" | ";" => { continue; }
                "action" => AstNode::Action(builder.get_action(&node)),
                "var_definition" => AstNode::VarDefinition(builder.get_var_definition(&node)),
                "definition" => AstNode::Definition(builder.get_definition(&node)),
                "sequence" => AstNode::Sequence(builder.get_sequence(&node)),
                x => panic!("error: unexpected top-level node: `{x}")
            };
            nodes.push((ast_node, Range::from(&node)));
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
        let contents = "
        load \"basics\";
        $positions := [(0,0),(1,1),(2,2)];
        move [$x] by $p := move $x by $p;
        // moves positions
        every frame move $positions by (10,10);";
        let tree = parse(contents);
        let root = tree.root_node();
        assert!(get!(root => sequence).has_error() == false);
        assert!(get!(root => var_definition).has_error() == false);
        assert!(get!(root => definition).has_error() == false);
        assert!(get!(root => comment).has_error() == false);
        assert!(get!(root => action).has_error() == false);
    }

    #[test]
    fn test_parse() {
        // empty file
        let ast = ast!("");
        assert!(ast.nodes.is_empty());

        // non-empty file
        let ast = ast!("
        load \"basics\";
        $positions := [(0,0),(1,1),(2,2)];
        move [$x] by $p := move $x by $p;
        // moves positions
        every frame move $positions by (10,10);
            ");
        assert_eq!(ast.nodes.len(), 4);
    }
}
