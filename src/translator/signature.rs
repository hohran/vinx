use std::fmt::Display;

use super::{Word, Sequence};
use crate::variable::VariableType;

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
