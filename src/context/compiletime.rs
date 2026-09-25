use crate::{translator::parser::{CompilationAction, Options}, variable::{Scope, Stack, Variable, VariableValue}};

pub struct Compiletime {
    stack: Stack,
    pub options: Options,
    actions: Vec<CompilationAction>,
}

impl Compiletime {
    pub fn new() -> Self {
        Self { stack: Stack::new(), options: Options::default(), actions: vec![] }
    }

    pub fn into_stack(self) -> Stack {
        self.stack
    }

    pub fn get_actions(&mut self) -> Vec<CompilationAction> {
        self.actions.drain(0..).collect()
    }

    pub fn add_action(&mut self, action: CompilationAction) {
        self.actions.push(action);
    }
}

impl super::Context for Compiletime {
    fn get_value<'c>(&'c self, v: &'c Variable) -> &'c VariableValue {
        v.get_value(&self.stack)
    }

    fn get_value_mut<'c>(&'c mut self, v: &'c mut Variable) -> &'c mut VariableValue {
        v.get_value_mut(&mut self.stack)
    }

    fn get_variable(&self, name: &str) -> Option<&VariableValue> {
        self.stack.get_variable(name)
    }

    fn set_value(&mut self, v: &mut Variable, new_value: VariableValue) {
        v.set_value(&mut self.stack, new_value);
    }

    fn update_variable(&mut self, name: &str, new_value: VariableValue) {
        self.stack.update_variable(name, new_value);
    }

    fn push_scope(&mut self) {
        self.stack.push();
    }

    fn push_scope_with(&mut self, scope: Scope) {
        self.stack.push_scope(scope);
    }

    fn pop_scope(&mut self) {
        self.stack.pop();
    }

    fn add_variable(&mut self, name: String, value: VariableValue) -> bool {
        self.stack.add_variable(name, value)
    }

    fn get_stack(&self) -> &Stack {
        &self.stack
    }

    fn get_stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }
}
