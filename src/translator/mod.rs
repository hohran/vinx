extern crate tree_sitter;
extern crate tree_sitter_vinx;

mod automata;
mod type_constraints;
mod translator;
pub mod word;
pub use word::Word;
use std::collections::HashMap;

pub use translator::parse;
use tree_sitter::Node;
mod builtins;
pub mod sequence;
// mod component_class;
mod operations;
mod structures;
mod value;
mod actions;
mod file_manager;

use crate::{context::Context, event::Operations, variable::{Stack, Structure, Variable, Scope, VariableType, VariableValue}};
// use crate::vtype;

#[derive(Debug)]
pub struct StructureTemplate {
    id: usize,
    // name: Option<String>,
    param_names: Vec<String>,
    param_types: Vec<VariableType>,
    members: Vec<(String, SequenceValue, Vec<Variable>)>,
}

impl StructureTemplate {
    pub fn new(id: usize, param_names: Vec<String>, param_types: Vec<VariableType>, members: Vec<(String, SequenceValue, Vec<Variable>)>) -> Self {
        Self { id, param_names, param_types, members }
    }

    pub fn instantiate(&self, params: Vec<Variable>, context: &mut Context, operations: &Operations, structures: &Vec<StructureTemplate>, stack: &mut Stack) -> Structure {
        assert_eq!(params.len(), self.param_names.len());
        // println!("instantiating structure {} with {params:?}", self.id);
        stack.push();
        let mut members = Scope::new();
        for i in 0..params.len() {
            assert!(params[i].get_type() == self.param_types[i]);
            members.insert(self.param_names[i].clone(), params[i].get_value(stack).clone());
            stack.add_variable(self.param_names[i].clone(), params[i].get_value(stack).clone());
        }
        for (name,val,ps) in &self.members {
            let member_val = match val {
                SequenceValue::Operation(id) => {
                    operations[*id]
                        .instantiate(ps.clone(), context, operations, structures, stack)
                        .process(context, stack, &mut HashMap::new(), operations) // TODO: fix hashmap for action activeness
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
        let s = Structure::new(self.id, members);
        s
    }
}

/// Gets node children without comments
fn get_children<'a>(node: &Node<'a>) -> Vec<Node<'a>> {
    node.named_children(&mut node.walk()).filter(|n| n.kind() != "comment").collect()
}

/// Gets node children with unnamed symbols without comments
fn get_all_children<'a>(node: &Node<'a>) -> Vec<Node<'a>> {
    node.children(&mut node.walk()).filter(|n| n.kind() != "comment").collect()
}


#[derive(Clone,Eq,PartialEq,Debug)]
pub enum SequenceValue {
    Operation(usize),
    Structure(usize),
    Value(VariableType),
}

impl SequenceValue {
    pub fn into_variable_type(&self, operations: &Operations) -> VariableType {
        match self {
            SequenceValue::Operation(f_id) => {
                let op = &operations[*f_id];
                let Some(ret) = op.get_return_type() else {
                    panic!("no return type for: {}", op.get_signature());
                };
                ret.clone()
            }
            SequenceValue::Structure(s) => VariableType::Structure(*s),
            SequenceValue::Value(t) => t.clone()
        }
    }
}
