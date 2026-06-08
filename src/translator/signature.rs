use std::fmt::Display;

use tree_sitter::Node;

use super::{Word, get_children, Sequence, Translator};
use crate::{translator::error::CompilationError, variable::{VariableType, VariableValue}};

/// Structure for handling signatures of operations and structures.
///
/// Example:
/// For signature: `move [$p] by $x`
///  - sequence: `move [Rectangle] by Pos`
///  - params: [$p,$x]
///  - iterators: [0] ($p)
///  - structure_param_id: None (only set for methods)
#[derive(Debug,Clone)]
pub struct Signature {
    pub sequence: Sequence,
    pub params: Vec<String>,
    pub iterators: Vec<usize>,
    pub structure_param_id: Option<usize>,
}

impl Signature {
    pub fn from(seq: Sequence) -> Self {
        Self { sequence: seq, params: vec![], iterators: vec![], structure_param_id: None }
    }

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

    /// Set the types of given signature to `types`.
    /// If a type should be an iterator, it is wrapped with a vector.
    pub fn swap_types(&mut self, types: &Vec<VariableType>) {
        let mut new_types = types.clone();
        for it in &self.iterators {
            new_types[*it] = VariableType::Vec(Box::new(types[*it].clone())); // TODO: make more effective
        }
        self.sequence.swap_types(&new_types);
    }

    /// Call `onParam` on every parameter (name,type)
    pub fn foreach<F>(&self, mut on_param: F) where F: FnMut(&str, &VariableType) {
        let types = self.sequence.get_types();
        for (i,param) in self.params.iter().enumerate() {
            if self.iterators.contains(&i) {
                on_param(param, types[i].unwrap_depth(1));
            } else {
                on_param(param, types[i]);
            }
        }
    }
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

impl Translator {
    /// Push every `signature` parameter to the stack, assigning it the default value of its type.
    pub fn push_signature_to_stack(&mut self, signature: &Signature) {
        signature.foreach(|p,t| if !self.globals.add_variable(p.to_string(), t.default()) { panic!("error: unexpected redeclaration of variables") } );
    }

    /// Update the type of every `signature` parameter on the stack.
    pub fn update_stack_with_signature(&mut self, signature: &Signature) {
        signature.foreach(|p,t| self.globals.update_variable(p, t.default()));
    }

    /// Get the signature represented by `node`.
    ///
    /// !!! IMPORTANT: this function automatically pushes the parameters to the stack, it is advised
    /// to create a dedicated scope beforehand !!!
    pub fn get_signature(&mut self, node: &Node) -> Result<Signature, CompilationError> {
        self.expect_node_kind(node, "signature");
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
                    sequence.push(Word::Type(VariableType::Any(param_id)));
                    params.push(param_name.to_string());
                    if param_name == structure_ref_name {
                        assert!(structure_param_id.is_none());  // TODO: friendlify
                        structure_param_id = Some(params.len()-1);
                    }
                    self.globals.add_variable(param_name.to_string(), VariableValue::Any(param_id));
                }
                "iterator" => {
                    let var_node = word_node.child_by_field_name("variable").expect("error: iterator without variable field");
                    self.expect_node_kind(&var_node, "variable");
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
                        if has_main_iterator {
                            return Err(CompilationError::MultipleMainIterators(self.get_location(node)));
                        }
                        has_main_iterator = true;
                        iterators.insert(0, var_id);
                    } else {
                        iterators.push(var_id);
                    }
                }
                x => panic!("error: unexpected type {x} in sequence")
            }
        }
        Ok(Signature { sequence, params, iterators, structure_param_id })
    }
}
