use std::fmt::Display;

use tree_sitter::Node;

use crate::{translator::{Word, get_children, sequence::Sequence, signature, translator::{Kind, Translator}}, variable::{VariableType, VariableValue}, word};

#[derive(Debug,Clone)]
pub struct Signature {
    pub sequence: Sequence,
    pub params: Vec<String>,
    pub iterators: Vec<usize>,
    pub structure_param_id: Option<usize>,
}

impl Display for Signature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut c = 0;
        for (i,w) in self.sequence.get().iter().enumerate() {
            if i != 0 { write!(f, " ")? }
            match w {
                Word::Keyword(k) => write!(f, "{k}")?,
                Word::Type(_) => {
                    if self.iterators.contains(&c) {
                        if self.iterators[0] == c {
                            write!(f, "[{}*]", self.params[c])?;
                        } else {
                            write!(f, "[{}]", self.params[c])?;
                        }
                    } else {
                        write!(f, "{}", self.params[c])?;
                    }
                    c += 1;
                }
            }
        }
        Ok(())
    }
}

impl Signature {
    /// For a method signature, set the id of the bound structure parameter
    pub fn set_structure_param(&mut self, structure_id: usize) {
        let Some(i) = self.structure_param_id else {
            panic!("error: signature {self} is not bound to a structure")
            // -- do we panic? or can we define methods not bound to structures?
        };
        if self.iterators.contains(&i) {
            self.sequence.swap_type_at(i, VariableType::Vec(Box::new(VariableType::Structure(structure_id))));
        } else {
            self.sequence.swap_type_at(i, VariableType::Structure(structure_id));
        }
    }

    pub fn swap_types(&mut self, types: &Vec<VariableType>) {
        let mut new_types = types.clone();
        for it in &self.iterators {
            new_types[*it] = VariableType::Vec(Box::new(types[*it].clone())); // TODO: make more effective
        }
        self.sequence.swap_types(&new_types);
    }
}

impl Translator {
    pub fn push_signature_to_stack(&mut self, signature: &Signature) {
        for (i,t) in signature.sequence.get_types().iter().enumerate() {
            self.globals.add_variable(signature.params[i].clone(), t.default());
        }
    }

    pub fn update_stack_with_signature(&mut self, signature: &Signature) {
        let types = signature.sequence.get_types();
        for (i,param) in signature.params.iter().enumerate() {
            if signature.iterators.contains(&i) {
                self.globals.update_variable(param, types[i].unwrap_depth(1).default());
            } else {
                self.globals.update_variable(param, types[i].default());
            }
        }
    }

    pub fn parse_signature(&mut self, node: &Node) -> Signature {
        node.expect_kind("signature", self);
        let structure_ref_name = "$self";
        let mut sequence = Sequence::new();
        let mut params = vec![];
        let mut has_main_iterator = false;
        let mut iterators = vec![];
        let mut structure_param_id = None;
        for word_node in get_children(node) {
            match word_node.kind() {
                "keyword" => sequence.push(Word::Keyword(self.text(&word_node).to_string())),
                "variable" => {
                    let param_id = self.new_unresolved_variable();
                    let param_name = self.get_variable_name(&word_node).unwrap();  // variable always has a valid name
                    sequence.push(word!(Any(param_id)));
                    params.push(param_name.to_string());
                    if param_name == structure_ref_name {
                        assert!(structure_param_id.is_none());  // TODO: friendlify
                        structure_param_id = Some(params.len()-1);
                    }
                    self.globals.add_variable(param_name.to_string(), VariableValue::Any(param_id));
                }
                "iterator" => {
                    let var_node = word_node.child_by_field_name("variable").expect("error: iterator without variable field");
                    var_node.expect_kind("variable", self);
                    let param_id = self.new_unresolved_variable();
                    let param_name = self.get_variable_name(&var_node).unwrap();  // variable always has a valid name
                    params.push(param_name.to_string());
                    if param_name == structure_ref_name {
                        assert!(structure_param_id.is_none());  // TODO: friendlify
                        structure_param_id = Some(params.len()-1);
                    }
                    sequence.push(Word::Type(VariableType::Any(param_id)));
                    self.globals.add_variable(param_name.to_string(), VariableValue::Any(param_id));
                    let var_id = self.globals.top().len()-1;
                    if word_node.child_by_field_name("main").is_some() {    // main iterator
                        assert!(has_main_iterator == false);    // TODO: nice error message / warning
                        has_main_iterator = true;
                        iterators.insert(0, var_id);
                    } else {
                        iterators.push(var_id);
                    }
                }
                x => panic!("error: unexpected type {x} in sequence")
            }
        }
        return Signature { sequence, params, iterators, structure_param_id }
    }
}
