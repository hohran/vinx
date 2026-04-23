use std::{collections::HashMap, fmt::Debug};

use crate::{context::Context, event::{Operations, builtins::Builtin, operation::Operation}, variable::{Variable, Stack, Scope, VariableValue}};

#[derive(Debug,Clone)]
pub enum EventEffect {
    Builtin(Builtin),
    Composed(Vec<Event>),
}

#[derive(Debug,Clone)]
pub struct Event {
    id: usize,
    params: Vec<Variable>,
    effect: EventEffect,
    vars: Scope,
    active_struct: bool,
}

impl Event {
    pub fn new(id: usize, params: Vec<Variable>, effect: EventEffect, vars: Scope) -> Self {
        Self { id, params, effect, vars, active_struct: true }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn deactivate_struct(&mut self) {
        self.active_struct = false;
    }

    pub fn process(&mut self, context: &mut Context, stack: &mut Stack, action_activeness: &mut HashMap<String,bool>, operations: &Operations) -> Option<VariableValue> {
        match &self.effect {
            EventEffect::Builtin(f) => f(context, stack, &mut self.params, action_activeness),
            EventEffect::Composed(_) => self.process_composed(context, stack, action_activeness, operations),
        }
    }

    fn process_composed(&mut self, context: &mut Context, stack: &mut Stack, action_activeness: &mut HashMap<String,bool>, operations: &Operations) -> Option<VariableValue> {
        let op = &operations[self.id];
        let iterators = op.get_iterators();
        let operands = op.get_operands();
        self.push_operation_layer(stack, op);
        self.push_structure_layer(stack, op);
        self.push_iterator_layer(stack, op, iterators);
        let iterations = self.get_iterations(iterators, stack);
        let iterated_params = self.get_iterated_params(iterators);
        let mut result = None;
        for it in 0..iterations {
            self.push_iterated_values(stack, &iterated_params, op, it);
            result = self.run_events(stack, context, action_activeness, operations);
            self.fetch_iterated_values(stack, &iterated_params, operands, it);
        }
        stack.pop();
        self.pop_structure_layer(stack, op);
        self.pop_operation_layer(stack, op);
        result
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
    fn get_iterations(&self, iterators: &Vec<usize>, stack: &mut Stack) -> usize {
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

    fn push_operation_layer(&self, stack: &mut Stack, op: &Operation) {
        assert!(self.params.len() == op.operands.len(), "error: incorrect number of parameters: expected {}, got {}", op.operands.len(), self.params.len());
        // stack.pretty_println("== operation layer ==".to_string());
        stack.push();
        for (n,v) in &self.vars {
            stack.add_variable(n.clone(), v.clone());
        }
        for i in 0..op.operands.len() {
            let val = self.params[i].get_value(&stack);
            stack.add_variable(op.operands[i].clone(), val.clone());
        }
    }

    fn pop_operation_layer(&mut self, stack: &mut Stack, op: &Operation) {
        assert!(self.params.len() == op.operands.len(), "error: incorrect number of parameters: expected {}, got {}", op.operands.len(), self.params.len());
        let layer = stack.pop();
        // stack.pretty_println("-- operation layer --".to_string());
        for i in 0..op.operands.len() {
            if op.structure_param_id == Some(i) {
                continue;
            }
            // stack.pretty_println(format!("getting var: {}", op.operands[i]));
            let val = layer.get(&op.operands[i]).unwrap();
            self.params[i].set_value(stack, val.clone());
        }
        let variable_names: Vec<String> = self.vars.keys().map(|x| x.clone()).collect();
        for var_name in variable_names {
            // stack.pretty_println(format!("getting var: {var_name}"));
            let val = layer.get(&var_name).unwrap();
            self.vars.insert(var_name, val.clone());
        }
    }

    fn push_structure_layer(&self, stack: &mut Stack, op: &Operation) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = op.structure_param_id {
            // stack.pretty_println("== structure layer ==".to_string());
            let VariableValue::Structure(s) = self.params[param_id].get_value(stack) else {
                panic!();
            };
            stack.push_scope(s.copy_members());
        }
    }

    fn pop_structure_layer(&mut self, stack: &mut Stack, op: &Operation) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = op.structure_param_id {
            let VariableValue::Structure(mut s) = self.params[param_id].get_value(stack).clone() else {
                panic!();
            };
            s.update(stack);
            self.params[param_id].set_value(stack, VariableValue::Structure(s));
            stack.pop();
        }
    }

    fn push_iterator_layer(&self, stack: &mut Stack, op: &Operation, iterators: &Vec<usize>) {
        // stack.pretty_println("== iterator layer ==".to_string());
        stack.push();
        for i in iterators {
            let it_name = &op.operands[*i];
            stack.add_variable(it_name.clone(), VariableValue::placeholder());
        }
    }

    fn run_events(&mut self, stack: &mut Stack, context: &mut Context, action_activeness: &mut HashMap<String,bool>, operations: &Operations) -> Option<VariableValue> {
        let mut result = None;
        let EventEffect::Composed(events) = &mut self.effect else {
            panic!("error: expected composed event");
        };
        for e in events {
            result = e.process(context, stack, action_activeness, operations);
        }
        result
    }

    fn push_iterated_values(&self, stack: &mut Stack, iterated_params: &Vec<bool>, op: &Operation, iteration: usize) {
        let param_values = self.get_param_values(stack);
        // update iterated values
        for i in 0..self.params.len() {
            if !iterated_params[i] {
                continue;
            }
            // get vector value
            let VariableValue::Vec(v) = &param_values[i] else {
                panic!("error: expected vector type for iterated value: {}, got {}", op.operands[i], param_values[i].get_type())
            };
            // get current iteration value
            let index = iteration % v.len();
            stack.update_variable(&op.operands[i], v[index].get_value(stack).clone());
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
