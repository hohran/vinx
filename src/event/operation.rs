use std::fmt::{Debug, Display};

use crate::{event::{Event, builtins::Builtin, event::{EventEffect, Operation}}, translator::{Sequence, Signature, parser::OperationMember}, variable::{Scope, Stack, Variable, VariableType}};

pub type Operations = Vec<OperationTemplateEnum>;

#[derive(Debug,Clone)]
pub struct OperationTemplate {
    id: usize,
    pub signature: Signature,
    effect: EventEffect,
    members: Vec<OperationMember>,
    result: Option<VariableType>,
}

#[derive(Debug, Clone, Copy)]
pub enum TopLevelOperation {
    LoadFile,
    DoNotSave,
}

pub enum OperationTemplateEnum {
    Standard(OperationTemplate),
    TopLevel(TopLevelOperation),
}

impl OperationTemplateEnum {
    pub fn get(&self) -> &OperationTemplate {
        let Self::Standard(op) = self else {
            panic!("error: tried to get a top-level operation"); // TODO: friendlify
        };
        op
    }
}

impl OperationTemplate {
    pub fn new(id: usize, signature: Signature, events: Vec<Event>, members: Vec<OperationMember>, result: Option<VariableType>) -> Self {
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

    pub fn instantiate(&self, params: Vec<Variable>) -> Operation {
        let Some(result) = &self.result else {
            return Operation::new(self.id, params, self.effect.clone(), None, Stack::scope_from_members(&self.members))
        };
        let return_type = if let Some(binding) = result.get_binding() {
            // find a parameter that has this binding
            // take the type from passed params
            let pos = self.signature.sequence.get_types().iter().position(|t| t.get_binding() == Some(binding)).unwrap();
            let type_depth = self.signature.sequence.get_types()[pos].get_depth();
            let mut param_type = params[pos].get_type();
            // we need to unwrap this type to what the binding represents
            // let's imagine that the signature is
            // `top [Any(0)]`
            // then we need to remove one level of depth from the passed parameter type
            // so `top [Color]` would imply the mapping Any(0) -> Color
            param_type = param_type.unwrap_depth(type_depth).clone();
            // finally we wrap the variable type of the binding to the actual depth of the return type
            // `make Any(0) a vector` with return type `[Any(0)]`
            // would mean that whatever type of parameter is passed, we need to wrap with one level
            // of depth.
            param_type.wrap_depth(result.get_depth());
            Some(param_type)
        } else {
            Some(result.clone())
        };
        Operation::new(self.id, params, self.effect.clone(), return_type, Stack::scope_from_members(&self.members))
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

impl Display for OperationTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.signature.params.len() == 0 {
            write!(f, "{}", self.signature.sequence)?;
        } else {
            write!(f, "{}", self.signature)?;
        }
        if let Some(return_type) = &self.result {
            write!(f, " => {return_type}")?;
        };
        Ok(())
    }
}
