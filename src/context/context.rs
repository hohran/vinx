use crate::variable::{Scope, Stack, Variable, VariableValue};

pub trait Context {
    fn get_value<'c>(&'c self, v: &'c Variable) -> &'c VariableValue;
    fn get_value_mut<'c>(&'c mut self, v: &'c mut Variable) -> &'c mut VariableValue;
    fn get_variable(&self, name: &str) -> Option<&VariableValue>;
    fn set_value(&mut self, var: &mut Variable, new_value: VariableValue);
    fn update_variable(&mut self, name: &str, new_value: VariableValue);
    fn push_scope(&mut self);
    fn push_scope_with(&mut self, scope: Scope);
    fn pop_scope(&mut self);
    fn add_variable(&mut self, name: String, value: VariableValue) -> bool;
    fn get_stack(&self) -> &Stack;
    fn get_stack_mut(&mut self) -> &mut Stack;
}
