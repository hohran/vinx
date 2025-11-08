mod operations;
pub mod component;
pub mod variable;

use std::collections::HashMap;

use component::Components;
use variable::{values::VariableValue, Variable, VariableLocation};

use crate::context::Context;
use operations::*;

#[derive(Debug,Clone)]
pub struct Event {
    id: usize,
    params: Vec<Variable>,
}

impl Event {
    pub fn new(id: usize, params: Vec<Variable>) -> Self {
        Self { id, params }
    }
}

impl Event {
    pub fn process(&self, context: &mut Context, scope: &mut HashMap<String, VariableValue>, components: &mut Components, action_activeness: &mut HashMap<String,bool>, operations: &HashMap<usize,Operation>) {
        match self.id {
            1 => move_pos(context, scope, &self.params),
            2 => move_pos_phase(context, scope, &self.params),
            3 => draw_rect(context, scope, &self.params),
            4 => set_activeness(context, scope, action_activeness, &self.params, true),
            5 => set_activeness(context, scope, action_activeness, &self.params, false),
            6 => set(context, scope, &self.params),
            7 => rotate_vec(context, scope, &self.params),
            8 => top_into(context, scope, &self.params),
            x => {
                let op = operations.get(&x).expect(&format!("error: unknown operation id {x}"));
                op.process(context, scope, components, action_activeness, operations, &self.params);
            }
        }
    }
}

#[derive(Debug,Clone)]
pub struct Operation {
    operands: Vec<String>,
    events: Vec<Event>,
}

pub type Operations = HashMap<usize,Operation>;

impl Operation {
    pub fn new(operands: Vec<String>, events: Vec<Event>) -> Self {
        Self { operands, events }
    }

    pub fn process(&self, context: &mut Context, scope: &mut HashMap<String,VariableValue>, components: &mut Components, action_activeness: &mut HashMap<String,bool>, operations: &Operations, params: &Vec<Variable>) {
        // ** load params
        assert!(params.len() == self.operands.len(), "error: incorrect number of parameters: expected {}, got {}", self.operands.len(), params.len());
        let mut tmp: Vec<Option<VariableValue>> = vec![];
        for i in 0..self.operands.len() {
            let val = &params[i].get_value(context, scope);
            let old = scope.insert(self.operands[i].clone(), val.clone());
            tmp.push(old);
        }
        // process events
        for e in &self.events {
            e.process(context, scope, components, action_activeness, operations);
        }
        // revert scope
        let mut updates: Vec<(String,VariableValue)> = vec![];
        for i in 0..self.operands.len() {
            let val = scope.remove(&self.operands[i]).unwrap();
            if let VariableLocation::Scope = params[i].get_location() {
                updates.push((params[i].get_name(),val));
            }
            if let Some(v) = tmp.remove(0) {
                scope.insert(self.operands[i].clone(), v);
            }
        }
        // update values
        for (name,val) in updates {
            scope.insert(name, val);
        }
    }
}
