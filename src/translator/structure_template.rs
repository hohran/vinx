use crate::context::Context;
use crate::translator::Signature;
use crate::translator::parser::{Call, Expression};
use crate::{context, variable::{Scope, Structure, VariableType}};

#[derive(Clone,Debug)]
pub struct StructureMember {
    name: String,
    value: Call,
}

impl StructureMember {
    pub fn new(name: String, value: Call) -> Self {
        Self { name, value }
    }

    pub fn instantiate_at_compiletime(&self, context: &mut context::Compiletime) {
        let value = self.value.evaluate_at_compiletime(context).unwrap();
        let has_collision = context.get_stack_mut().add_variable(self.name.clone(), value.clone());
        assert!(!has_collision);
    }

    pub fn instantiate_at_runtime(&self, context: &mut context::Runtime) {
        let value = self.value.evaluate_at_runtime(context).unwrap();
        let has_collision = context.get_stack_mut().add_variable(self.name.clone(), value.clone());
        assert!(!has_collision);
    }
}

// TODO refactor
#[derive(Clone,Debug)]
pub struct StructureTemplate {
    id: usize,
    param_names: Vec<String>,
    param_types: Vec<VariableType>,
    members: Vec<StructureMember>,
    signature: Signature,
}

impl PartialEq for StructureTemplate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl StructureTemplate {
    pub fn new(id: usize, param_names: Vec<String>, param_types: Vec<VariableType>, members: Vec<StructureMember>, signature: Signature) -> Self {
        Self { id, param_names, param_types, members, signature }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_signature(&self) -> &Signature {
        &self.signature
    }

    pub fn evaluate_at_compiletime(&self, params: &Vec<Expression>, context: &mut context::Compiletime) -> Structure {
        assert_eq!(params.len(), self.param_names.len());
        let mut members = Scope::new();
        context.push_scope(); {
            for i in 0..params.len() {
                let val = params[i].evaluate_at_compiletime(context).unwrap();
                assert!(val.get_type().is_assignable_to(&self.param_types[i]));
                members.insert(self.param_names[i].clone(), context.get_value(&val).clone());
                context.add_variable(self.param_names[i].clone(), context.get_value(&val).clone()); // TODO: we should cast it to the expected param type (self.params[i])
            }
            for member in &self.members {
                member.instantiate_at_compiletime(context);
            }
        } context.pop_scope();
        let s = Structure::new(self.id, members);
        s
    }

    pub fn evaluate_at_runtime(&self, params: &Vec<Expression>, context: &mut context::Runtime) -> Structure {
        assert_eq!(params.len(), self.param_names.len());
        let mut members = Scope::new();
        context.push_scope(); {
            for i in 0..params.len() {
                let val = params[i].evaluate_at_runtime(context).unwrap(); // FIXME: unwrap
                assert!(val.get_type().is_assignable_to(&self.param_types[i]));
                members.insert(self.param_names[i].clone(), context.get_value(&val).clone());
                context.add_variable(self.param_names[i].clone(), context.get_value(&val).clone());
            }
            for member in &self.members {
                member.instantiate_at_runtime(context);
            }
        } context.pop_scope();
        let s = Structure::new(self.id, members);
        s
    }
}
