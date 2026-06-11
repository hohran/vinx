mod ast;
mod value;
mod file_load;
pub mod sequence;
pub mod signature;
mod action;
pub mod definition;
mod var_definition;
mod builder;
mod range;

use builder::AstBuilder;
pub use ast::{Ast, AstNode};
pub use signature::{Signature, Iterator};
pub use action::{Action, Trigger, Time, Unit, Event};
pub use definition::Definition;
pub use var_definition::{VarDefinition,Assignment};
pub use sequence::Sequence;
pub use value::Value;

#[derive(Debug, Clone, Copy)]
pub struct Range (tree_sitter::Point, tree_sitter::Point);
fn get_range(node: &tree_sitter::Node) -> Range {
    let r = node.range();
    Range(r.start_point, r.end_point)
}
