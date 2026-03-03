mod operations;

use std::{collections::HashMap, fmt::Display};

use crate::{context::Context, translator::{SequenceValue, StructureTemplate, Word, seq_to_str}, variable::{Variable, stack::{Stack, VariableMap}, values::VariableValue}};
use operations::*;

#[derive(Debug,Clone)]
pub struct Event {
    id: usize,
    params: Vec<Variable>,
    events: Vec<Event>,
    vars: VariableMap,
    active_struct: bool,
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
        Self { id, params, events, vars, active_struct: true }
    }

    pub fn deactivate_struct(&mut self) {
        self.active_struct = false;
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn process(&mut self, context: &mut Context, scope: &mut Stack, action_activeness: &mut HashMap<String,bool>, operations: &HashMap<usize,Operation>) {
        match self.id {
            1 => move_pos(context, scope, &mut self.params),
            2 => move_pos_phase(context, scope, &mut self.params),
            3 => draw_rect_outline(context, scope, &mut self.params),
            4 => set_activeness(scope, action_activeness, &mut self.params, true),
            5 => set_activeness(scope, action_activeness, &mut self.params, false),
            6 => set(scope, &mut self.params),
            7 => rotate_vec(scope, &mut self.params),
            8 => top_into(scope, &mut self.params),
            9 => add(scope, &mut self.params),
            10 => draw_rect(context, scope, &mut self.params),
            11 => draw_effect_rect(context, scope, &mut self.params),
            12 => toggle_activeness(scope, action_activeness, &mut self.params),
            13 => sub(scope, &mut self.params),
            14 => move_by(scope, &mut self.params),
            15 => print(scope, &self.params),
            _ => {
                self.process_operation(context, scope, action_activeness, operations);
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
    fn get_iterations(&self, iterators: &Vec<usize>, scope: &mut Stack) -> usize {
        if iterators.is_empty() {
            1
        } else {
            let main_iter = iterators[0];
            let VariableValue::Vec(v) = self.params[main_iter].get_value(scope) else {panic!()};
            v.len()
        }
    }

    fn process_operation(&mut self, context: &mut Context, scope: &mut Stack, action_activeness: &mut HashMap<String,bool>, operations: &HashMap<usize,Operation>) {
        // println!("\t!!! PROCESS OPERTAION !!!");
        let op = operations.get(&self.id).expect(&format!("error: unknown operation {}", self.id));
        // scope.pretty_println(format!("'{}'", seq_to_str(&op.signature)));
        let iterators = op.get_iterators();
        let operands = op.get_operands();
        self.push_operation_layer(scope, op);
        self.push_structure_layer(scope, op);
        self.push_iterator_layer(scope, op, iterators);
        let iterations = self.get_iterations(iterators, scope);
        let iterated_params = self.get_iterated_params(iterators);
        for it in 0..iterations {
            self.push_iterated_values(scope, &iterated_params, op, it);
            self.run_events(scope, context, action_activeness, operations);
            self.fetch_iterated_values(scope, &iterated_params, operands, it);
        }
        scope.pop_layer();
        // scope.pretty_println("-- iterator layer --".to_string());
        self.pop_structure_layer(scope, op);
        self.pop_operation_layer(scope, op);
    }

    fn push_operation_layer(&self, scope: &mut Stack, op: &Operation) {
        assert!(self.params.len() == op.operands.len(), "error: incorrect number of parameters: expected {}, got {}", op.operands.len(), self.params.len());
        // scope.pretty_println("== operation layer ==".to_string());
        scope.push_layer();
        for (n,v) in &self.vars {
            scope.add_variable(n.clone(), v.clone());
        }
        for i in 0..op.operands.len() {
            let val = &self.params[i].get_value(&scope);
            scope.add_variable(op.operands[i].clone(), val.clone());
        }
    }

    fn pop_operation_layer(&mut self, scope: &mut Stack, op: &Operation) {
        assert!(self.params.len() == op.operands.len(), "error: incorrect number of parameters: expected {}, got {}", op.operands.len(), self.params.len());
        let layer = scope.pop_layer();
        // scope.pretty_println("-- operation layer --".to_string());
        for i in 0..op.operands.len() {
            if op.structure_param_id == Some(i) {
                continue;
            }
            // scope.pretty_println(format!("getting var: {}", op.operands[i]));
            let val = layer.get(&op.operands[i]).unwrap();
            self.params[i].set_value(scope, val.clone());
        }
        let variable_names: Vec<String> = self.vars.keys().map(|x| x.clone()).collect();
        for var_name in variable_names {
            // scope.pretty_println(format!("getting var: {var_name}"));
            let val = layer.get(&var_name).unwrap();
            self.vars.insert(var_name, val.clone());
        }
    }

    fn push_structure_layer(&self, scope: &mut Stack, op: &Operation) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = op.structure_param_id {
            // scope.pretty_println("== structure layer ==".to_string());
            scope.push_layer();
            let VariableValue::Structure(s) = self.params[param_id].get_value(scope) else {
                panic!();
            };
            s.populate_stack(scope);
        }
    }

    fn pop_structure_layer(&mut self, scope: &mut Stack, op: &Operation) {
        if !self.active_struct {
            return;
        }
        if let Some(param_id) = op.structure_param_id {
            let VariableValue::Structure(mut s) = self.params[param_id].get_value(scope) else {
                panic!();
            };
            s.update(scope);
            self.params[param_id].set_value(scope, VariableValue::Structure(s));
            scope.pop_layer();
            // scope.pretty_println("-- structure layer --".to_string());
        }
    }

    fn push_iterator_layer(&self, scope: &mut Stack, op: &Operation, iterators: &Vec<usize>) {
        // scope.pretty_println("== iterator layer ==".to_string());
        scope.push_layer();
        for i in iterators {
            let it_name = &op.operands[*i];
            scope.add_variable(it_name.clone(), VariableValue::empty());
        }
    }

    fn run_events(&mut self, scope: &mut Stack, context: &mut Context, action_activeness: &mut HashMap<String,bool>, operations: &HashMap<usize,Operation>) {
        for e in self.events.iter_mut() {
            e.process(context, scope, action_activeness, operations);
        }
    }

    fn push_iterated_values(&self, scope: &mut Stack, iterated_params: &Vec<bool>, op: &Operation, iteration: usize) {
        let param_values = self.get_param_values(scope);
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
            let index = iteration % v.len();
            scope.update_variable(&op.operands[i], v[index].get_value(scope));
        }
    }

    fn fetch_iterated_values(&self, scope: &mut Stack, iterated_params: &Vec<bool>, operands: &Vec<String>, iteration: usize) {
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
                scope.update_vec_at(&operands[i], iteration % v.len(), new_val.clone(),&val.get_type());
            }
        }
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

    fn get_param_values(&self, scope: &mut Stack) -> Vec<VariableValue> {
        self.params.iter().map(|x| x.get_value_of_type(scope, x.get_type())).collect()
    }
}


pub type Operations = HashMap<usize,Operation>;

#[derive(Debug,Clone)]
pub struct Operation {
    id: usize,
    signature: Vec<Word>,
    operands: Vec<String>,
    events: Vec<Event>,
    iterators: Vec<usize>,
    members: Vec<(String,SequenceValue,Vec<Variable>)>,
    structure_param_id: Option<usize>,
}

impl Operation {
    pub fn new(id: usize, signature: Vec<Word>, operands: Vec<String>, events: Vec<Event>, iterators: Vec<usize>, members: Vec<(String,SequenceValue,Vec<Variable>)>, structure_param_id: Option<usize>) -> Self {
        Self { id, operands, events, iterators, members, structure_param_id, signature }
    }

    /// Returns the respective structure
    pub fn method_of(&self) -> Option<&usize> {
        self.structure_param_id.as_ref()
    }

    pub fn instantiate(&self, params: Vec<Variable>, structures: &Vec<StructureTemplate>, stack: &mut Stack) -> Event {
        // println!("instantiate op {} with {params:?}", self.id);
        stack.push_layer();
        for i in 0..params.len() {
            let name = self.operands[i].clone();
            let value = params[i].get_value(stack);
            stack.add_variable(name, value);
        }
        let mut members = VariableMap::new();
        for (name,val,ps) in &self.members {
            let member_val = match val {
                SequenceValue::Operation(_) => {
                    todo!("operation return values");
                }
                SequenceValue::Component(id) => {
                    let val = structures[*id].instantiate(ps.clone(), structures, stack);
                    VariableValue::Structure(val)
                    // members.insert(name.clone(), VariableValue::Structure(val));
                }
                SequenceValue::Value(_) => {
                    assert_eq!(ps.len(), 1, "only 1 param for value");
                    ps[0].get_value(stack)
                    // members.insert(name.clone(), ps[0].get_value(stack));
                }
            };
            members.insert(name.clone(), member_val.clone());
            stack.add_variable(name.clone(), member_val);
        }
        stack.pop_layer();
        Event { id: self.id, params, events: self.events.clone(), vars: members, active_struct: true }
    }

    pub fn push_to_stack(&self, params: &Vec<Variable>, variables: &VariableMap, stack: &mut Stack) {
        assert!(params.len() == self.operands.len(), "error: incorrect number of parameters: expected {}, got {}", self.operands.len(), params.len());
        stack.push_layer_with(variables.clone());
        for i in 0..self.operands.len() {
            let val = &params[i].get_value(stack);
            stack.add_variable(self.operands[i].clone(), val.clone());
        }
    }

    pub fn poop_from_stack(&self, params: &mut Vec<Variable>, variables: &mut VariableMap, stack: &mut Stack, structure_param_id: Option<usize>) {
        assert!(params.len() == self.operands.len(), "error: incorrect number of parameters: expected {}, got {}", self.operands.len(), params.len());
        let layer = stack.pop_layer();
        for i in 0..self.operands.len() {
            if structure_param_id != Some(i) {
                let val = layer.get(&self.operands[i]).unwrap();
                params[i].set_value(stack, val.clone());
            }
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
