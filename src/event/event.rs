use std::fmt::Debug;

use crate::{context, event::{Event, builtins::{Builtin, BuiltinCompiletime, BuiltinRuntime}, operation::OperationTemplate}, translator::parser::Expression, variable::{Scope, Stack, Variable, VariableType, VariableValue}};
use context::Context;

#[derive(Clone,PartialEq,Debug)]
pub enum Func {
    Builtin(Builtin),
    RuntimeBuiltin(BuiltinRuntime),
    CompiletimeBuiltin(BuiltinCompiletime),
}

#[derive(Clone,PartialEq,Debug)]
pub enum EventEffect {
    Builtin(Func),
    Composed(Vec<Event>),
}

#[derive(Clone,PartialEq,Debug)]
pub struct Operation {
    template: OperationTemplate,
    params: Vec<Expression>,
    effect: EventEffect,
    vars: Scope,
    active_struct: bool,
    return_type: Option<VariableType>,
}

impl Operation {
    pub fn new(params: Vec<Expression>, effect: EventEffect, return_type: Option<VariableType>, vars: Scope, template: OperationTemplate) -> Self {
        Self { params, effect, vars, active_struct: true, return_type, template }
    }

    pub fn get_return_type(&self) -> Option<&VariableType> {
        self.return_type.as_ref()
    }

    pub fn get_id(&self) -> usize {
        self.template.get_id()
    }

    pub fn deactivate_struct(&mut self) {
        self.active_struct = false;
    }

    /// Evaluate all parameters, pass them to be processed, and update them (when possible)...
    /// should we really update?.
    pub fn process_at_compiletime<'a>(&mut self, context: &mut context::Compiletime) -> Option<VariableValue> {
        // evaluate params
        let mut params: Vec<Variable> = self.params.iter()
            .map(|expr| {
                let v = expr.evaluate_at_compiletime(context);
                match v {
                    Some(v) => v,
                    None => panic!("error: failed to evaluate expression: {expr:?}"),
                }
            }).collect();
        // process
        let output = match &self.effect {
            EventEffect::Builtin(func) => match func {
                Func::Builtin(f) => f(context, &mut params),
                Func::CompiletimeBuiltin(f) => f(context, &mut params),
                Func::RuntimeBuiltin(_) => panic!("error: tried to call runtime function at compiletime: `{}`", self.template.get_signature())
            }
            EventEffect::Composed(_) => todo!("composed functions at compiletime"),
        };
        // update params
        // for (i, new_value) in params.into_iter().enumerate() {
        //     let Expression::Static(value) = &mut self.params[i] else { continue };
        //     *value = new_value;
        // }
        // return the output
        output
    }

    pub fn process<'a, 'b: 'a>(&mut self, context: &'a mut context::Runtime<'b>) -> Option<VariableValue> {
        // evaluate params
        let mut params: Vec<Variable> = self.params.iter()
            .map(|expr| {
                let v = expr.evaluate_at_runtime(context);
                match v {
                    Some(v) => v,
                    None => panic!("error: failed to evaluate expression: {expr:?}"),
                }
            }).collect();
        // process
        let output = match &self.effect {
            EventEffect::Builtin(func) => match func {
                Func::Builtin(f) => f(context, &mut params),
                Func::RuntimeBuiltin(f) => f(context, &mut params),
                Func::CompiletimeBuiltin(_) => panic!("error: tried to call compiletime function at runtime: `{}`", self.template.get_signature())
            }
            EventEffect::Composed(_) => self.process_composed(&mut params, context),
        };
        // update params
        // for (i, new_value) in params.into_iter().enumerate() {
        //     let Expression::Static(value) = &mut self.params[i] else { continue };
        //     *value = new_value;
        // }
        // return the output
        output
    }

    fn process_composed(&mut self, params: &mut Vec<Variable>, context: &mut context::Runtime) -> Option<VariableValue> {
        self.push_layers(params, context);
        let iterated_params = self.get_iterated_params();
        let mut result = None;
        for it in self.get_iterations_(params, context) {
            self.push_iterated_values(params, context.get_stack_mut(), &iterated_params, it);
            result = self.run_events(context);
            self.fetch_iterated_values(params, context.get_stack_mut(), &iterated_params, it);
        }
        self.pop_layers(params, context);
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
    fn push_layers(&mut self, params: &mut Vec<Variable>, context: &mut dyn context::Context) {
        // it is important to fetch the operation layer before pushing the structure layer.
        // if a parameter would have the same name and type as a structure member, it would be
        // overwritten otherwise because of the way the variable value is evaluated.
        let operation_layer = self.get_operation_layer(params, context.get_stack());
        self.push_structure_layer(params, context.get_stack_mut()); // having layers in this order makes sure that method parameters override structure members
        context.push_scope_with(operation_layer);
        self.push_iterator_layer(context.get_stack_mut());
    }

    fn pop_layers(&mut self, params: &mut Vec<Variable>, context: &mut dyn context::Context) {
        context.pop_scope();
        self.pop_operation_layer(params, context.get_stack_mut());
        self.pop_structure_layer(params, context.get_stack_mut());
    }

    fn get_operation_layer(&self, params: &mut Vec<Variable>, stack: &Stack) -> Scope {
        let mut scope = self.vars.clone();
        let op_params = self.template.get_params();
        for i in 0..op_params.len() {
            let val = params[i].get_value(&stack);
            scope.insert(op_params[i].clone(), val.clone());
        }
        scope
    }

    /// params: Int, [Pos], [Pos], [Int]
    /// iterators: [3,1]
    /// returns [F,T,F,T]
    fn get_iterated_params(&self) -> Vec<bool> {
        let iterators = self.template.get_iterators();
        let mut ret = vec![false; self.params.len()];
        for it in iterators {
            ret[*it] = true;
        }
        ret
    }

    /// Get number of iterations, i.e., the length of the main iterator.
    fn get_iterations(&self, params: &mut Vec<Variable>, stack: &Stack) -> usize {
        let iterators = self.template.get_iterators();
        if iterators.is_empty() {
            1
        } else {
            let main_iter = iterators[0];
            let VariableValue::Vec(v) = params[main_iter].get_value(stack) else {
                panic!("error: iterator is not a vector");
            };
            v.len()
        }
    }

    fn get_iterations_(&self, params: &mut Vec<Variable>, context: &impl context::Context) -> std::ops::Range<usize> {
        let iterators = self.template.get_iterators();
        if iterators.is_empty() {
            0..1
        } else {
            let main_iter = iterators[0];
            let VariableValue::Vec(v) = context.get_value(&params[main_iter]) else {
                panic!("error: iterator is not a vector");
            };
            0..v.len()
        }
    }

    fn pop_operation_layer(&mut self, params: &mut Vec<Variable>, stack: &mut Stack) {
        let op_params = self.template.get_params();
        assert!(self.params.len() == op_params.len(), "error: incorrect number of parameters: expected {}, got {}", op_params.len(), self.params.len());
        let layer = stack.pop();
        for i in 0..op_params.len() {
            if self.template.method_of() == Some(&i) {
                continue;
            }
            let val = layer.get(&op_params[i]).unwrap();
            params[i].set_value(stack, val.clone());
        }
        let variable_names: Vec<String> = self.vars.keys().map(|x| x.clone()).collect();
        for var_name in variable_names {
            let val = layer.get(&var_name).unwrap();
            self.vars.insert(var_name, val.clone());
        }
    }

    fn push_structure_layer(&self, params: &mut Vec<Variable>, stack: &mut Stack) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = self.template.method_of() {
            // FIXME: vec of structures is not allowed
            let VariableValue::Structure(s) = params[*param_id].get_value(stack) else {
                panic!();
            };
            stack.push_scope(s.copy_members());
        }
    }

    fn pop_structure_layer(&mut self, params: &mut Vec<Variable>, stack: &mut Stack) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = self.template.method_of() {
            let VariableValue::Structure(mut s) = params[*param_id].get_value(stack).clone() else {
                panic!();
            };
            s.update(stack);
            params[*param_id].set_value(stack, VariableValue::Structure(s));
            stack.pop();
        }
    }

    fn push_iterator_layer(&self, stack: &mut Stack) {
        stack.push();
        let op_params = self.template.get_params();
        for i in self.template.get_iterators() {
            let it_name = &op_params[*i];
            stack.add_variable(it_name.clone(), VariableValue::placeholder());
        }
    }

    fn run_events(&mut self, context: &mut context::Runtime) -> Option<VariableValue> {
        let mut result = None;
        let EventEffect::Composed(events) = &mut self.effect else {
            panic!("error: expected composed event");
        };
        for e in events {
            result = e.process(context);
        }
        result
    }

    // fn push_iterated_values(&self, stack: &mut Stack, iterated_params: &Vec<bool>, op: &OperationTemplate, iteration: usize) {
    //     let param_values = self.get_param_values(stack);
    //     // update iterated values
    //     for i in 0..self.params.len() {
    //         if !iterated_params[i] {
    //             continue;
    //         }
    //         // get vector value
    //         let VariableValue::Vec(v) = &param_values[i] else {
    //             panic!("error: expected vector type for iterated value: {}, got {}", op.get_params()[i], param_values[i].get_type())
    //         };
    //         // get current iteration value
    //         let index = iteration % v.len();
    //         stack.update_variable(&op.get_params()[i], v[index].get_value(stack).clone());
    //     }
    // }

    fn push_iterated_values(&self, params: &mut Vec<Variable>, stack: &mut Stack, iterated_params: &Vec<bool>, iteration: usize) {
        let op_params = self.template.get_params();
        let param_values = self.get_param_values(params, stack);
        // update iterated values
        for i in 0..self.params.len() {
            if !iterated_params[i] {
                continue;
            }
            // get vector value
            let VariableValue::Vec(v) = &param_values[i] else {
                panic!("error: expected vector type for iterated value: {}, got {}", op_params[i], param_values[i].get_type())
            };
            // get current iteration value
            let index = iteration % v.len();
            stack.update_variable(&op_params[i], v[index].get_value(stack).clone());
        }
    }

    // fn fetch_iterated_values(&self, stack: &mut Stack, iterated_params: &Vec<bool>, operands: &Vec<String>, iteration: usize) {
    //     for i in 0..self.params.len() {
    //         if !iterated_params[i] {
    //             continue;
    //         }
    //         // get iterated value from stack
    //         let new_val = stack.get_variable(&operands[i]).expect(&format!("error: operand {} without value", operands[i]));
    //         // update vector
    //         let v = stack.get_variable_of_type(&operands[i], &self.params[i].get_type());
    //         if let Some(val) = v {
    //             let VariableValue::Vec(v) = val else {panic!()};
    //             stack.update_vec_at(&operands[i], iteration % v.len(), new_val.clone(),&val.get_type());
    //         }
    //     }
    // }

    fn fetch_iterated_values(&self, params: &mut Vec<Variable>, stack: &mut Stack, iterated_params: &Vec<bool>, iteration: usize) {
        let operands = self.template.get_params();
        for i in 0..self.params.len() {
            if !iterated_params[i] {
                continue;
            }
            // get iterated value from stack
            let new_val = stack.get_variable(&operands[i]).expect(&format!("error: operand {} without value", operands[i]));
            // update vector
            let v = stack.get_variable_of_type(&operands[i], &params[i].get_type());
            if let Some(val) = v {
                let VariableValue::Vec(v) = val else {panic!()};
                stack.update_vec_at(&operands[i], iteration % v.len(), new_val.clone(),&val.get_type());
            }
        }
    }

    // fn get_param_values(&self, stack: &mut Stack) -> Vec<VariableValue> {
    //     self.params.iter().map(|x| x.get_value(stack).clone()).collect()
    // }

    fn get_param_values(&self, params: &mut Vec<Variable>, stack: &mut Stack) -> Vec<VariableValue> {
        params.iter().map(|x| x.get_value(stack).clone()).collect()
    }
}
