use tree_sitter::Node;

pub struct AstBuilder {
    filename: String,
    source_code: String,
}

impl AstBuilder {
    pub fn new(filename: &str, source_code: String) -> Self {
        Self { filename: filename.to_string(), source_code }
    }

    pub fn expect_node_kind(&self, node: &Node, expect: &str) {
        let kind = node.kind();
        if kind != expect {
            let start = node.range().start_point;
            panic!("{}:{}:{}: error: expected node type to be `{}`, got `{}`",self.filename, start.row, start.column, expect, kind);
        }
    }

    /// Get the corresponding source code of the node.
    pub fn text(&self, node: &Node) -> &str {
        let range = node.range();
        &self.source_code[range.start_byte..range.end_byte]
    }
}
