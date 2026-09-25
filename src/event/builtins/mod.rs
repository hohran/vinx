use crate::variable::{Variable, VariableValue, Direction};
use crate::context;

pub mod general;
pub mod compiletime;
pub mod runtime;

pub use general::Builtin;
pub use compiletime::BuiltinCompiletime;
pub use runtime::BuiltinRuntime;

fn expect_param_count(operation_name: &str, params: &Vec<Variable>, expected: usize) {
    assert_eq!(params.len(), expected, "error: function {operation_name} expected {expected} parameters, got {}", params.len());
}
