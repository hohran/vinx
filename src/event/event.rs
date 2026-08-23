use std::fmt::Debug;

use crate::{context, event::{Event, Operations, builtins::{Builtin, BuiltinRuntime}, operation::OperationTemplate}, variable::{Scope, Stack, Variable, VariableType, VariableValue}};
use context::Context;

#[derive(Debug,Clone)]
pub enum Func {
    Builtin(Builtin),
    RuntimeBuiltin(BuiltinRuntime),
}

#[derive(Debug,Clone)]
pub enum EventEffect {
    Builtin(Func),
    Composed(Vec<Event>),
}

#[derive(Debug,Clone)]
pub struct Operation {
    id: usize,
    params: Vec<Variable>,
    effect: EventEffect,
    vars: Scope,
    active_struct: bool,
    return_type: Option<VariableType>,
}

impl Operation {
    pub fn new(id: usize, params: Vec<Variable>, effect: EventEffect, return_type: Option<VariableType>, vars: Scope) -> Self {
        Self { id, params, effect, vars, active_struct: true, return_type }
    }

    pub fn get_return_type(&self) -> Option<&VariableType> {
        self.return_type.as_ref()
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn deactivate_struct(&mut self) {
        self.active_struct = false;
    }

    pub fn process_at_compiletime<'a>(&mut self, context: &mut context::Compiletime, operations: &Operations) -> Option<VariableValue> {
        match &self.effect {
            EventEffect::Builtin(func) => match func {
                Func::Builtin(f) => f(context, &mut self.params),
                Func::RuntimeBuiltin(_) => panic!("error: tried to call runtime function at compiletime: `{}`", operations[self.id].get().signature)
            }
            EventEffect::Composed(_) => todo!("composed functions at compiletime"),
        }
    }

    pub fn process<'a, 'b: 'a>(&mut self, context: &'a mut context::Runtime<'b>, operations: &Operations) -> Option<VariableValue> {
        match &self.effect {
            EventEffect::Builtin(func) => match func {
                Func::Builtin(f) => f(context, &mut self.params),
                Func::RuntimeBuiltin(f) => f(context, &mut self.params),
            }
            EventEffect::Composed(_) => self.process_composed(context, operations),
        }
    }

    fn process_composed(&mut self, context: &mut context::Runtime, operations: &Operations) -> Option<VariableValue> {
        let op = &operations[self.id].get();
        let iterators = op.get_iterators();
        let operands = op.get_params();
        self.push_layers(context, op);
        let iterations = self.get_iterations(iterators, context.get_stack());
        let iterated_params = self.get_iterated_params(iterators);
        let mut result = None;
        for it in 0..iterations {
            self.push_iterated_values(context.get_stack_mut(), &iterated_params, op, it);
            result = self.run_events(context, operations);
            self.fetch_iterated_values(context.get_stack_mut(), &iterated_params, operands, it);
        }
        self.pop_layers(context, op);
        result
    }

    /// Push all the layers (scopes) necessary for this operation.
    /// The layers are pushed in this order:
    ///  * Structure layer
    ///    - created when the operation is a method to a structure
    ///    - necessary to access the structure members
    ///  * Operation layer
    ///    - this layer contains all operation parameters and members
    ///  * Iteration layer
    ///    - this layer shadows all iterated parameters with placeholder values, which are then
    ///    repopulated in each iteration
    ///
    /// The order dictates, that structure variables can be shadowed by the operation variables.
    fn push_layers(&mut self, context: &mut dyn context::Context, op: &OperationTemplate) {
        // we need to fetch the operation layer before pushing the structure because
        // it is possible that some structure variable actually shadows the value of the parameter
        //
        // ```vinx
        // /* global variable */
        // ...
        // $color := green
        // $color $rectangle := {
        //   ...
        //   change $self color to $new_color := {
        //     $color = $new_color;
        //   }
        // }
        // $r := red $rect;
        // at $x frames { 
        //   /* 
        //     in this operation, we need to cache the value of the global $color, because it is
        //     inaccessible after pushing the structure layer (containing a variable with same name
        //     and type)
        //   */
        //   change color of $r to $color;
        // }
        let operation_layer = self.get_operation_layer(context.get_stack(), op);
        self.push_structure_layer(context.get_stack_mut(), op); // having layers in this order makes sure that method parameters override structure members
        context.push_scope_with(operation_layer);
        self.push_iterator_layer(context.get_stack_mut(), op, op.get_iterators());
    }

    fn pop_layers(&mut self, context: &mut dyn context::Context, op: &OperationTemplate) {
        context.pop_scope();
        self.pop_operation_layer(context.get_stack_mut(), op);
        self.pop_structure_layer(context.get_stack_mut(), op);
    }

    fn get_operation_layer(&self, stack: &Stack, op: &OperationTemplate) -> Scope {
        let mut scope = self.vars.clone();
        for i in 0..op.get_params().len() {
            let val = self.params[i].get_value(&stack);
            scope.insert(op.get_params()[i].clone(), val.clone());
        }
        scope
    }

    /// params: Int, [Pos], [Pos], [Int]
    /// iterators: [3,1]
    /// returns [F,T,F,T]
    fn get_iterated_params(&self, iterators: &Vec<usize>) -> Vec<bool> {
        let mut ret = vec![false; self.params.len()];
        for it in iterators {
            ret[*it] = true;
        }
        ret
    }

    /// Get number of iterations, i.e., the length of the main iterator.
    fn get_iterations(&self, iterators: &Vec<usize>, stack: &Stack) -> usize {
        if iterators.is_empty() {
            1
        } else {
            let main_iter = iterators[0];
            let VariableValue::Vec(v) = self.params[main_iter].get_value(stack) else {
                panic!("error: iterator is not a vector");
            };
            v.len()
        }
    }

    fn push_operation_layer(&self, stack: &mut Stack, op: &OperationTemplate) {
        assert!(self.params.len() == op.get_params().len(), "error: incorrect number of parameters: expected {}, got {}", op.get_params().len(), self.params.len());
        stack.push();
        for (n,v) in &self.vars {
            stack.add_variable(n.clone(), v.clone());
        }
        for i in 0..op.get_params().len() {
            let val = self.params[i].get_value(&stack);
            stack.add_variable(op.get_params()[i].clone(), val.clone());
        }
    }

    fn pop_operation_layer(&mut self, stack: &mut Stack, op: &OperationTemplate) {
        assert!(self.params.len() == op.get_params().len(), "error: incorrect number of parameters: expected {}, got {}", op.get_params().len(), self.params.len());
        let layer = stack.pop();
        // stack.pretty_println("-- operation layer --".to_string());
        for i in 0..op.get_params().len() {
            if op.method_of() == Some(&i) {
                continue;
            }
            // stack.pretty_println(format!("getting var: {}", op.operands[i]));
            let val = layer.get(&op.get_params()[i]).unwrap();
            self.params[i].set_value(stack, val.clone());
        }
        let variable_names: Vec<String> = self.vars.keys().map(|x| x.clone()).collect();
        for var_name in variable_names {
            // stack.pretty_println(format!("getting var: {var_name}"));
            let val = layer.get(&var_name).unwrap();
            self.vars.insert(var_name, val.clone());
        }
    }

    fn push_structure_layer(&self, stack: &mut Stack, op: &OperationTemplate) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = op.method_of() {
            // FIXME: vec of structures is not allowed
            let VariableValue::Structure(s) = self.params[*param_id].get_value(stack) else {
                panic!();
            };
            stack.push_scope(s.copy_members());
        }
    }

    fn pop_structure_layer(&mut self, stack: &mut Stack, op: &OperationTemplate) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = op.method_of() {
            let VariableValue::Structure(mut s) = self.params[*param_id].get_value(stack).clone() else {
                panic!();
            };
            s.update(stack);
            self.params[*param_id].set_value(stack, VariableValue::Structure(s));
            stack.pop();
        }
    }

    fn push_iterator_layer(&self, stack: &mut Stack, op: &OperationTemplate, iterators: &Vec<usize>) {
        stack.push();
        for i in iterators {
            let it_name = &op.get_params()[*i];
            stack.add_variable(it_name.clone(), VariableValue::placeholder());
        }
    }

    fn run_events(&mut self, context: &mut context::Runtime, operations: &Operations) -> Option<VariableValue> {
        let mut result = None;
        let EventEffect::Composed(events) = &mut self.effect else {
            panic!("error: expected composed event");
        };
        for e in events {
            result = e.process(context, operations);
        }
        result
    }

    fn push_iterated_values(&self, stack: &mut Stack, iterated_params: &Vec<bool>, op: &OperationTemplate, iteration: usize) {
        let param_values = self.get_param_values(stack);
        // update iterated values
        for i in 0..self.params.len() {
            if !iterated_params[i] {
                continue;
            }
            // get vector value
            let VariableValue::Vec(v) = &param_values[i] else {
                panic!("error: expected vector type for iterated value: {}, got {}", op.get_params()[i], param_values[i].get_type())
            };
            // get current iteration value
            let index = iteration % v.len();
            stack.update_variable(&op.get_params()[i], v[index].get_value(stack).clone());
        }
    }

    fn fetch_iterated_values(&self, stack: &mut Stack, iterated_params: &Vec<bool>, operands: &Vec<String>, iteration: usize) {
        for i in 0..self.params.len() {
            if !iterated_params[i] {
                continue;
            }
            // get iterated value from stack
            let new_val = stack.get_variable(&operands[i]).expect(&format!("error: operand {} without value", operands[i]));
            // update vector
            let v = stack.get_variable_of_type(&operands[i], &self.params[i].get_type());
            if let Some(val) = v {
                let VariableValue::Vec(v) = val else {panic!()};
                stack.update_vec_at(&operands[i], iteration % v.len(), new_val.clone(),&val.get_type());
            }
        }
    }

    fn get_param_values(&self, stack: &mut Stack) -> Vec<VariableValue> {
        self.params.iter().map(|x| x.get_value(stack).clone()).collect()
    }
}
