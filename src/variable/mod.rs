mod types;
mod values;
mod stack;
mod variable;

pub use stack::{VariableMap,Stack};
pub use types::VariableType;
pub use variable::Variable;
pub use values::{VariableValue,Direction,Structure,Effect,Color};
