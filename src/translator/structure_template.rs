use super::*;
use crate::context::Context;
use crate::{context, event::Operations, translator::parser::StructureMember, variable::{Scope, Structure, Variable, VariableType, VariableValue}};

// TODO refactor
#[derive(Debug)]
pub struct StructureTemplate {
    id: usize,
    param_names: Vec<String>,
    param_types: Vec<VariableType>,
    members: Vec<StructureMember>,
}

impl StructureTemplate {
    pub fn new(id: usize, param_names: Vec<String>, param_types: Vec<VariableType>, members: Vec<(String, SequenceValue, Vec<Variable>)>) -> Self {
        Self { id, param_names, param_types, members }
    }

    pub fn instantiate(&self, params: Vec<Variable>, context: &mut context::Compiletime, operations: &Operations, structures: &Vec<StructureTemplate>) -> Structure {
        assert_eq!(params.len(), self.param_names.len());
        context.push_scope();
        let mut members = Scope::new();
        for i in 0..params.len() {
            assert!(params[i].get_type().is_assignable_to(&self.param_types[i]));
            members.insert(self.param_names[i].clone(), context.get_value(&params[i]).clone());
            context.add_variable(self.param_names[i].clone(), context.get_value(&params[i]).clone()); // TODO: we should cast it to the expected param type (self.params[i])
        }
        for (name,val,ps) in &self.members {
            let member_val = match val {
                SequenceValue::Operation(id) => {
                    operations[*id].get()
                        .instantiate(ps.clone())
                        .process_at_compiletime(context, operations) // TODO: fix hashmap for action activeness
                        .expect("error: did not have value")
                }
                SequenceValue::Structure(id) => {
                    let val = structures[*id].instantiate(ps.clone(), context, operations, structures);
                    VariableValue::Structure(val)
                }
            };
            members.insert(name.clone(), member_val.clone());
            context.add_variable(name.clone(), member_val);
        }
        context.pop_scope();
        let s = Structure::new(self.id, members);
        s
    }
}
