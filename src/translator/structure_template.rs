use super::*;
use crate::{context::Context, event::Operations, translator::parser::StructureMember, variable::{Scope, Stack, Structure, Variable, VariableType, VariableValue}};

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

    pub fn instantiate(&self, params: Vec<Variable>, context: &mut Context, operations: &Operations, structures: &Vec<StructureTemplate>, stack: &mut Stack) -> Structure {
        assert_eq!(params.len(), self.param_names.len());
        stack.push();
        let mut members = Scope::new();
        for i in 0..params.len() {
            assert!(params[i].get_type().is_assignable_to(&self.param_types[i]));
            members.insert(self.param_names[i].clone(), params[i].get_value(stack).clone());
            stack.add_variable(self.param_names[i].clone(), params[i].get_value(stack).clone()); // TODO: we should cast it to the expected param type (self.params[i])
        }
        for (name,val,ps) in &self.members {
            let member_val = match val {
                SequenceValue::Operation(id) => {
                    operations[*id].get()
                        .instantiate(ps.clone())
                        .process(context, stack, &mut vec![], operations) // TODO: fix hashmap for action activeness
                        .expect("error: did not have value")
                }
                SequenceValue::Structure(id) => {
                    let val = structures[*id].instantiate(ps.clone(), context, operations, structures, stack);
                    VariableValue::Structure(val)
                }
            };
            members.insert(name.clone(), member_val.clone());
            stack.add_variable(name.clone(), member_val);
        }
        stack.pop();
        let s = Structure::new(self.id, members);
        s
    }
}
