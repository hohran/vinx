use super::*;

#[derive(Debug, Clone, Copy)]
pub struct Range (tree_sitter::Point, tree_sitter::Point);
impl Range {
    pub fn new(start: tree_sitter::Point, end: tree_sitter::Point) -> Self {
        Self(start, end)
    }

    pub fn default() -> Self {
        Self(tree_sitter::Point { row: 0, column: 0 }, tree_sitter::Point { row: 0, column: 0 })
    }

    pub fn start_point(&self) -> &tree_sitter::Point {
        &self.0
    }

    pub fn end_point(&self) -> &tree_sitter::Point {
        &self.1
    }
}

fn get_range(node: &tree_sitter::Node) -> Range {
    let r = node.range();
    Range(r.start_point, r.end_point)
}

impl From<&tree_sitter::Node<'_>> for Range {
    fn from(value: &tree_sitter::Node) -> Self {
        let r = value.range();
        Self(r.start_point, r.end_point)
    }
}

impl From<&Sequence> for Range {
    fn from(value: &Sequence) -> Self {
        assert!(value.len() > 0);
        let start = value[0].1.start_point();
        let end = value[value.len()-1].1.end_point();
        Self(*start, *end)
    }
}

impl From<&Signature> for Range {
    fn from(value: &Signature) -> Self {
        assert!(value.len() > 0);
        let start = value[0].1.start_point();
        let end = value[value.len()-1].1.end_point();
        Self(*start, *end)
    }
}

impl From<&Definition> for Range {
    fn from(value: &Definition) -> Self {
        let start = value.signature[0].1.start_point();
        let body = &value.body;
        let end = if body.len() == 0 {
            value.signature[0].1.end_point()
        } else {
            body[body.len()-1].1.end_point()
        };
        Self(*start, *end)
    }
}
