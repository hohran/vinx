use crate::translator::parser::CompilationAction;

use super::*;
use context::Context;

/// Function callable only at compiletime.
/// Such function can access global options of the whole program.
pub type BuiltinCompiletime = fn(&mut context::Compiletime, &mut Vec<Variable>) -> Option<VariableValue>;

pub fn load_file(context: &mut context::Compiletime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("load file", params, 1);

    let filename = context.get_value(&params[0]).to_string();
    context.add_action(CompilationAction::LoadFile(filename));
    None
}

pub fn do_not_save(context: &mut context::Compiletime, params: &mut Vec<Variable>) -> Option<VariableValue> {
    expect_param_count("do not save", params, 0);
    context.options.save_video = false;
    None
}
