extern crate tree_sitter;
extern crate tree_sitter_vinx;

mod automata;
mod type_constraints;
mod translator;
mod word;
mod signature;
mod error;
mod builtins;
mod sequence;
mod operations;
mod structures;
mod value;
mod actions;
mod file_manager;
pub mod ast;

pub mod parser;

pub use translator::Translator;
pub use translator::parse;
pub use signature::Signature;
pub use sequence::{SequenceValue, Sequence};
pub use operations::MemberDef;
pub use structures::StructureTemplate;

use automata::Automaton;
use type_constraints::TypeConstraints;
use word::Word;
use sequence::{OperationId, StructureId};
use error::{Warning, CompilationError, Location};
use builtins::{load_builtin_operations, load_builtin_structures};
use file_manager::FileManager;

use tree_sitter::Node;

/// Gets node children without comments
fn get_children<'a>(node: &Node<'a>) -> Vec<Node<'a>> {
    node.named_children(&mut node.walk()).filter(|n| n.kind() != "comment").collect()
}

/// Gets node children with unnamed symbols without comments
fn get_all_children<'a>(node: &Node<'a>) -> Vec<Node<'a>> {
    node.children(&mut node.walk()).filter(|n| n.kind() != "comment").collect()
}
