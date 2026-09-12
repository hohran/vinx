use std::fmt::{Debug, Display};

use crate::{event::{Event, event::{EventEffect, Func, Operation}}, translator::{Sequence, Signature, parser::{Expression, OperationMember}}, variable::{Scope, Stack, Variable, VariableType}};

pub type Operations = Vec<OperationTemplateEnum>;

#[derive(Clone,PartialEq,Debug)]
pub struct OperationTemplate {
    id: usize,
    pub signature: Signature,
    effect: EventEffect,
    members: Vec<OperationMember>,
    result: Option<VariableType>,
}

#[derive(Debug,PartialEq, Clone, Copy)]
pub enum TopLevelOperation {
    LoadFile,
    DoNotSave,
}

#[derive(Clone,PartialEq,Debug)]
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
        Self::check_return_type(signature.sequence.get_types(), &result);
        Self { id, effect: EventEffect::Composed(events), members, signature, result }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn from_builtin(id: usize, sequence: Sequence, builtin: Func, result: Option<VariableType>) -> Self {
        Self::check_return_type(sequence.get_types(), &result);
        Self { id, signature: Signature::from(sequence), effect: EventEffect::Builtin(builtin), members: vec![], result }
    }

    /// if the return type is ambiguous, we need to check, that its binding is present in the
    /// parameters.
    ///
    /// `process [Any(0)] -> Any(1)` is a problematic signature, because it returns ambiguous value
    /// even for a concrete input.
    fn check_return_type(params: Vec<&VariableType>, return_type: &Option<VariableType>) {
        if let Some(Some(return_type_binding)) = return_type.as_ref().map(|t| t.get_binding()) {
            assert!(params.iter().any(|t| t.get_binding() == Some(return_type_binding)));
        }
    }

    pub fn get_signature(&self) -> &Signature {
        &self.signature
    }

    pub fn get_return_type(&self) -> Option<&VariableType> {
        self.result.as_ref()
    }

    pub fn get_signature_sequence(&self) -> &Sequence {
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

    pub fn compute_return_type(&self, params: &Vec<Variable>) -> Option<VariableType> {
        self.result.as_ref().map(|r| self.signature.sequence.compute_return_type(r, &params))
    }

    pub fn ___compute_return_type(&self, params: &Vec<VariableType>) -> Option<VariableType> {
        self.result.as_ref().map(|r| self.signature.sequence.___compute_return_type(r, &params))
    }

    pub fn instantiate(&self, params: Vec<Expression>) -> Operation {
        let return_type = self.___compute_return_type(&params.iter().map(|expr| expr.get_type()).collect());
        Operation::new(params, self.effect.clone(), return_type, Stack::scope_from_members(&self.members), self.clone())
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
