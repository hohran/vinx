use std::fmt::Debug;

use crate::{context::Context, event::{builtins::Builtin, event::{Event, EventEffect}}, translator::{MemberDef, SequenceValue, Signature, StructureTemplate, Sequence}, variable::{Scope, Stack, Variable, VariableType, VariableValue}};

pub type Operations = Vec<OperationTemplate>;

#[derive(Debug,Clone)]
pub struct OperationTemplate {
    id: usize,
    pub signature: Signature,
    effect: EventEffect,
    members: Vec<(String,SequenceValue,Vec<Variable>)>,
    result: Option<VariableType>,
}

impl OperationTemplate {
    pub fn new(id: usize, signature: Signature, events: Vec<Event>, members: Vec<MemberDef>, result: Option<VariableType>) -> Self {
        Self { id, effect: EventEffect::Composed(events), members, signature, result }
    }

    pub fn from_builtin(id: usize, sequence: Sequence, builtin: Builtin, result: Option<VariableType>) -> Self {
        Self { id, signature: Signature::from(sequence), effect: EventEffect::Builtin(builtin), members: vec![], result }
    }

    pub fn get_return_type(&self) -> Option<&VariableType> {
        self.result.as_ref()
    }

    pub fn get_signature(&self) -> &Sequence {
        &self.signature.sequence
    }

    pub fn is_iterated(&self) -> bool {
        !self.signature.iterators.is_empty()
    }

    /// Returns if the operation is a structure method.
    pub fn is_method(&self) -> bool {
        self.signature.structure_param_id.is_some()
    }

    /// Returns the respective structure
    pub fn method_of(&self) -> Option<&usize> {
        self.signature.structure_param_id.as_ref()
    }

    pub fn instantiate(&self, params: Vec<Variable>, context: &mut Context, operations: &Operations, structures: &Vec<StructureTemplate>, stack: &mut Stack) -> Event {
        // println!("instantiate op {} with {params:?}", self.id);
        if self.members.is_empty() {
            return Event::new(self.id, params, self.effect.clone(), Scope::new());
        }
        stack.push();
        for i in 0..params.len() {
            let name = self.signature.params[i].clone();
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
        assert!(params.len() == self.signature.params.len(), "error: incorrect number of parameters: expected {}, got {}", self.signature.params.len(), params.len());
        stack.push_scope(variables.clone());
        for i in 0..self.signature.params.len() {
            let val = params[i].get_value(stack);
            stack.add_variable(self.signature.params[i].clone(), val.clone());
        }
    }

    pub fn get_iterators(&self) -> &Vec<usize> {
        &self.signature.iterators
    }

    pub fn get_params(&self) -> &Vec<String> {
        &self.signature.params
    }

    pub fn get_iterated_param_name(&self, param_index: usize) -> String {
        format!("{}!", self.signature.params[param_index])
    }
}
