use std::fmt::Debug;

use crate::{context::Context, event::{builtins::Builtin, event::{Event, EventEffect}}, translator::{SequenceValue, StructureTemplate, sequence::Sequence}, variable::{Variable, Stack, Scope, VariableType, VariableValue}};

pub type Operations = Vec<Operation>;

#[derive(Debug,Clone)]
pub struct Operation {
    id: usize,
    signature: Sequence,
    pub operands: Vec<String>,
    effect: EventEffect,
    iterators: Vec<usize>,
    members: Vec<(String,SequenceValue,Vec<Variable>)>,
    pub structure_param_id: Option<usize>,
    result: Option<VariableType>,
}

impl Operation {
    pub fn new(id: usize, signature: Sequence, operands: Vec<String>, events: Vec<Event>, iterators: Vec<usize>, members: Vec<(String,SequenceValue,Vec<Variable>)>, structure_param_id: Option<usize>, result: Option<VariableType>) -> Self {
        Self { id, operands, effect: EventEffect::Composed(events), iterators, members, structure_param_id, signature, result }
    }

    pub fn from_builtin(id: usize, signature: Sequence, builtin: Builtin, result: Option<VariableType>) -> Self {
        Self { id, signature, operands: vec![], effect: EventEffect::Builtin(builtin), iterators: vec![], members: vec![], structure_param_id: None, result }
    }

    pub fn get_return_type(&self) -> Option<&VariableType> {
        self.result.as_ref()
    }

    pub fn get_signature(&self) -> &Sequence {
        &self.signature
    }

    pub fn is_iterated(&self) -> bool {
        !self.iterators.is_empty()
    }

    /// Returns if the operation is a structure method.
    pub fn is_method(&self) -> bool {
        self.structure_param_id.is_some()
    }

    /// Returns the respective structure
    pub fn method_of(&self) -> Option<&usize> {
        self.structure_param_id.as_ref()
    }

    pub fn instantiate(&self, params: Vec<Variable>, context: &mut Context, operations: &Operations, structures: &Vec<StructureTemplate>, stack: &mut Stack) -> Event {
        // println!("instantiate op {} with {params:?}", self.id);
        if self.members.is_empty() {
            return Event::new(self.id, params, self.effect.clone(), Scope::new());
        }
        stack.push();
        for i in 0..params.len() {
            let name = self.operands[i].clone();
            let value = params[i].get_value(stack).clone();
            stack.add_variable(name, value);
        }
        let mut members = Scope::new();
        // TODO: refactor
        for (name,val,ps) in &self.members {
            let member_val = match val {
                SequenceValue::Operation(id) => {
                    operations[*id]
                        .instantiate(ps.clone(), context, operations, structures, stack)
                        .process(context, stack, &mut vec![], operations)
                        .expect("error: did not have value")
                }
                SequenceValue::Structure(id) => {
                    let val = structures[*id].instantiate(ps.clone(), context, operations, structures, stack);
                    VariableValue::Structure(val)
                }
                SequenceValue::Value(_) => {
                    assert_eq!(ps.len(), 1, "only 1 param for value");
                    ps[0].get_value(stack).clone()
                }
            };
            members.insert(name.clone(), member_val.clone());
            stack.add_variable(name.clone(), member_val);
        }
        stack.pop();
        Event::new(self.id, params, self.effect.clone(), members)
    }

    pub fn push_to_stack(&self, params: &Vec<Variable>, variables: &Scope, stack: &mut Stack) {
        assert!(params.len() == self.operands.len(), "error: incorrect number of parameters: expected {}, got {}", self.operands.len(), params.len());
        stack.push_scope(variables.clone());
        for i in 0..self.operands.len() {
            let val = params[i].get_value(stack);
            stack.add_variable(self.operands[i].clone(), val.clone());
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
