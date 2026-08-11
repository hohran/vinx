macro_rules! get {
    ( $node:expr $(=> $child:tt)* ) => {
        {
            let mut node = $node;
            $(node = node.children(&mut node.walk()).find(|n| n.kind() == stringify!($child)).unwrap();)*
                node
        }
    };
}

macro_rules! ast {
    ( $x:expr ) => { Ast::parse_from_contents("tmp.vinx", $x.to_string()) };
}

pub fn parse(contents: &str) -> tree_sitter::Tree {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_vinx::LANGUAGE.into()).expect("error: could not load vinx grammar");
    parser.parse(&contents, None).unwrap()
}

pub(crate) use get;
pub(crate) use ast;
