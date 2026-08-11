extern crate tree_sitter;
extern crate tree_sitter_vinx;

mod automata;
mod type_constraints;
mod word;
mod signature;
mod error;
mod builtins;
mod sequence;
mod structure_template;
mod file_manager;
pub mod ast;

pub mod parser;

pub use signature::Signature;
pub use sequence::{SequenceValue, Sequence};
pub use structure_template::StructureTemplate;

use word::Word;
