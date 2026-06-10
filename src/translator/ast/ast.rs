use super::{Action, VarDefinition, Definition, AstBuilder};

pub enum AstNode {
    Action(Action),
    VarDefinition(VarDefinition),
    Definition(Definition),
    FileLoad(String),
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
        let mut nodes = vec![];
        let builder = AstBuilder::new(filepath, contents.to_string());
        for node in root_node.children(&mut root_node.walk()) {
            match node.kind() {
                "comment" => {}
                "action" => nodes.push(AstNode::Action(builder.get_action(&node))),
                "var_definition" => nodes.push(AstNode::VarDefinition(builder.get_var_definition(&node))),
                "definition" => nodes.push(AstNode::Definition(builder.get_definition(&node))),
                "file_load" => nodes.push(AstNode::FileLoad(builder.get_file_load(&node))),
                x => panic!("error: unexpected top-level node: `{x}")
            }
        }
        Ast { nodes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! ast {
        ( $x:expr ) => { Ast::parse_from_contents("tmp.vinx", $x.to_string()) };
    }

    #[test]
    fn test_parse() {
        // empty file
        let ast = ast!("");
        assert!(ast.nodes.is_empty());

        // non-empty file
        let ast = ast!("
        load \"basics\";
        $positions = [(0,0),(1,1),(2,2)];
        move [$x] by $p = move $x by $p;
        // moves positions
        every frame move $positions by (10,10);
            ");
        assert_eq!(ast.nodes.len(), 4);
    }
}
