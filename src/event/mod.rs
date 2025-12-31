mod operations;
pub mod component;

use std::{collections::HashMap, fmt::Display};

use component::Components;

use crate::{context::Context, variable::{stack::{Stack, VariableMap}, values::VariableValue, Variable, VariableLocation}};
use operations::*;

#[derive(Debug,Clone)]
pub struct Event {
    id: usize,
    params: Vec<Variable>,
    events: Vec<Event>,
    vars: VariableMap,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<String> = self.params.iter().map(|x| x.to_string()).collect();
        let ev_strs: Vec<String> = self.events.iter().map(|x| x.to_string()).collect();
        let events = if ev_strs.is_empty() { "".to_string() } else { format!("{{\n  {}\n}}", ev_strs.join("\n  ")) };
        write!(f, "event {} with params ({}) {}", self.id, params.join(","), events)
    }
}

impl Event {
    pub fn new(id: usize, params: Vec<Variable>, events: Vec<Event>, vars: VariableMap) -> Self {
        Self { id, params, events, vars }
    }

    pub fn process(&mut self, context: &mut Context, scope: &mut Stack, components: &mut Components, action_activeness: &mut HashMap<String,bool>, operations: &HashMap<usize,Operation>) {
        match self.id {
            1 => move_pos(context, scope, &mut self.params),
            2 => move_pos_phase(context, scope, &mut self.params),
            3 => draw_rect_outline(context, scope, &mut self.params),
            4 => set_activeness(context, scope, action_activeness, &mut self.params, true),
            5 => set_activeness(context, scope, action_activeness, &mut self.params, false),
            6 => set(context, scope, &mut self.params),
            7 => rotate_vec(context, scope, &mut self.params),
            8 => top_into(context, scope, &mut self.params),
            9 => add(context, scope, &mut self.params),
            10 => draw_rect(context, scope, &mut self.params),
            11 => draw_effect_rect(context, scope, &mut self.params),
            12 => toggle_activeness(context, scope, action_activeness, &mut self.params),
            _ => {
                self.process_operation(context, scope, components, action_activeness, operations);
            }
        }
    }

    /// params: Int, [Pos], [Pos], [Int]
    /// iterators: [3,1]
    /// returns [F,T,F,T]
    fn get_iterated_params(&self, iterators: &Vec<usize>) -> Vec<bool> {
        let mut ret = vec![false;self.params.len()];
        for it in iterators {
            ret[*it] = true;
        }
        ret
    }

    /// [] -> 1 iteration
    /// [3,1,2] -> 3 iterations
    fn get_iterations(&self, iterators: &Vec<usize>, scope: &mut Stack, context: &mut Context) -> usize {
        if iterators.is_empty() {
            1
        } else {
            let main_iter = iterators[0];
            let VariableValue::Vec(v) = self.params[main_iter].get_value(context, scope) else {panic!()};
            v.len()
        }
    }

    fn process_operation(&mut self, context: &mut Context, scope: &mut Stack, components: &mut Components, action_activeness: &mut HashMap<String,bool>, operations: &HashMap<usize,Operation>) {
        // println!("\t!!! PROCESS OPERTAION !!!");
        let op = operations.get(&self.id).expect(&format!("error: unknown operation {}", self.id));
        let iterators = op.get_iterators();
        let operands = op.get_operands();
        op.push_to_stack(&self.params, &self.vars, context, scope); 
        let iterations = self.get_iterations(iterators, scope, context);
        let iterated_params = self.get_iterated_params(iterators);
        // load iterated names
        scope.push_layer();
        for i in iterators {
            let it_name = &op.operands[*i];
            scope.add_variable(it_name.clone(), VariableValue::empty());
        }
        // println!("number of iterations: {iterations}");
        for it in 0..iterations {
            let param_values = self.get_param_values(context, scope);
            // update iterated values
            for i in 0..self.params.len() {
                if !iterated_params[i] {
                    continue;
                }
                // get vector value
                let VariableValue::Vec(v) = &param_values[i] else {
                    panic!("error: expected vector type for iterated value: {}, got {}", op.operands[i], param_values[i].get_type().to_string())
                };
                // get current iteration value
                let index = it % v.len();
                scope.update_variable(&op.operands[i], v[index].get_value(context, scope));
            }
            // run events
            for e in self.events.iter_mut() {
                e.process(context, scope, components, action_activeness, operations);
            }
            // save iterated values
            for i in 0..self.params.len() {
                if !iterated_params[i] {
                    continue;
                }
                // get iterated value from scope
                let new_val = scope.get_variable(&operands[i]).expect(&format!("error: operand {} without value", operands[i]));
                // update vector
                let v = scope.get_variable_of_type(&operands[i], self.params[i].get_type());
                if let Some(val) = v {
                    let VariableValue::Vec(v) = val else {panic!()};
                    scope.update_vec_at(&operands[i], it % v.len(), new_val.clone(),&val.get_type());
                }
                // let index = it % v.len();
                // let changed_var = &v[index];
                // v[index].set_value(context, scope, val.clone());
            }
        }
        scope.pop_layer();
        op.pop_from_stack(&mut self.params, &mut self.vars, context, scope);
    }

    pub fn to_string_with_indent(&self, indent: usize) -> String {
        let ind_str = format!("{:1$}", ' ', indent);
        let params: Vec<String> = self.params.iter().map(|x| x.to_string()).collect();
        let ev_strs: Vec<String> = self.events.iter().map(|x| x.to_string_with_indent(indent+2)).collect();
        let events = if ev_strs.is_empty() { 
            "".to_string() 
        } else { 
            format!(":\n{ind_str}{}", ev_strs.join(&format!("\n{ind_str}")))
        };
        format!("event {} with params ({}){}", self.id, params.join(","), events)
    }

    fn apply_iterators(&self, iterators: &Vec<usize>, context: &mut Context, scope: &mut Stack) -> Vec<Vec<Variable>> {
        if iterators.is_empty() {
            return vec![self.params.clone()];
        }
        let param_values = self.get_param_values(context, scope);
        let main_iterator_id = iterators[0];
        let VariableValue::Vec(v) = &param_values[main_iterator_id] else {
            panic!("error: {:?} is not an iterator", self.params[main_iterator_id]);
        };
        let len = v.len();
        let mut ret = vec![vec![];len];
        for iteration in 0..len {
            for var_id in 0..param_values.len() {
                let val = if iterators.contains(&var_id) {
                    let VariableValue::Vec(v) = &param_values[var_id] else {
                        panic!();
                    };
                    v[len % v.len()].clone()
                } else {
                    self.params[var_id].clone()
                };
                ret[iteration].push(val);
            }
        }
        ret
    }

    fn get_param_values(&self, context: &mut Context, scope: &mut Stack) -> Vec<VariableValue> {
        self.params.iter().map(|x| x.get_value_of_type(context, scope, x.get_type())).collect()
    }
}


pub type Operations = HashMap<usize,Operation>;

#[derive(Debug,Clone)]
pub struct Operation {
    id: usize,
    operands: Vec<String>,
    events: Vec<Event>,
    iterators: Vec<usize>,
    variables: VariableMap, // static values for each operation
}

impl Operation {
    pub fn new(id: usize, operands: Vec<String>, events: Vec<Event>, iterators: Vec<usize>, variables: VariableMap) -> Self {
        Self { id, operands, events, iterators, variables }
    }

    pub fn instantiate(&self, params: Vec<Variable>) -> Event {
        Event { id: self.id, params, events: self.events.clone(), vars: self.variables.clone() }
    }

    pub fn push_to_stack(&self, params: &Vec<Variable>, variables: &VariableMap, context: &Context, stack: &mut Stack) {
        assert!(params.len() == self.operands.len(), "error: incorrect number of parameters: expected {}, got {}", self.operands.len(), params.len());
        stack.push_layer_with(variables.clone());
        for i in 0..self.operands.len() {
            let val = &params[i].get_value(context, stack);
            stack.add_variable(self.operands[i].clone(), val.clone());
        }
    }

    pub fn pop_from_stack(&self, params: &mut Vec<Variable>, variables: &mut VariableMap, context: &Context, stack: &mut Stack) {
        assert!(params.len() == self.operands.len(), "error: incorrect number of parameters: expected {}, got {}", self.operands.len(), params.len());
        let layer = stack.pop_layer();
        for i in 0..self.operands.len() {
            let val = layer.get(&self.operands[i]).unwrap();
            if self.operands[i] == "$x" {
                // println!("... setting value to: {}", val.to_string());
            }
            params[i].set_value(context, stack, val.clone());
        }
        let variable_names: Vec<String> = variables.keys().map(|x| x.clone()).collect();
        for var_name in variable_names {
            let val = layer.get(&var_name).unwrap();
            variables.insert(var_name, val.clone());
        }
    }

    pub fn get_iterators(&self) -> &Vec<usize> {
        &self.iterators
    }

    pub fn get_operands(&self) -> &Vec<String> {
        &self.operands
    }

    pub fn get_iterated_param_name(&self, param_index: usize) -> String {
        format!("{}!", self.operands[param_index])
    }
}
