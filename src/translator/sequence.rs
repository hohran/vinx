use std::fmt::Display;

use super::{Word, StructureTemplate};
use crate::{context, event::{OperationTemplateEnum, Operations, TopLevelOperation}, translator::{Signature, error::Location, parser::Expression}, variable::{Variable, VariableType, VariableValue}};

pub type OperationId = usize;
pub type StructureId = usize;

// #[derive(Clone,Eq,PartialEq,Debug,Copy,Hash)]
// pub enum SequenceType {
//     Operation,
//     Structure,
// }
//
// impl SequenceType {
//     pub fn to_value(self, id: usize) -> SequenceValue {
//         match self {
//             Self::Operation => SequenceValue::Operation(id),
//             Self::Structure => SequenceValue::Structure(id),
//         }
//     }
// }

// TODO: refactor
#[derive(Clone,PartialEq,Debug)]
pub enum SequenceValue {
    Operation(OperationTemplateEnum),
    Structure(StructureTemplate),
}

impl SequenceValue {
    /// Return a variable type of given sequence value.
    /// This type can be ambiguous! For this reason, this function is named the way it is.
    ///
    /// For example for operation:
    /// `top $vec`, where $vec: [Any(0)], the return type would be `Any(0)`
    ///
    /// To have a concrete return type, you need to instantiate the operation with parameters first,
    /// to be able to infer it.
    /// Structure should always return the concrete type.
    pub fn get_general_return_type(&self) -> VariableType {
        match self {
            SequenceValue::Operation(op) => {
                let Some(ret) = op.get().get_return_type() else {
                    panic!("no return type for: {}", op.get().get_signature_sequence());
                };
                ret.clone()
            }
            SequenceValue::Structure(s) => VariableType::Structure(s.get_id()),
        }
    }

    pub fn get_return_type(&self, params: &Vec<VariableType>) -> Option<VariableType> {
        match self {
            SequenceValue::Operation(op) => {
                op.get().___compute_return_type(params)
            }
            SequenceValue::Structure(s) => Some(VariableType::Structure(s.get_id())),
        }
    }

    pub fn get_signature(&self) -> &Signature {
        match self {
            SequenceValue::Structure(s) => s.get_signature(),
            Self::Operation(op) => &op.get().get_signature(),
        }
    }

    pub fn evaluate_at_compiletime(&self, params: &Vec<Expression>, context: &mut context::Compiletime) -> Option<VariableValue> {
        match self {
            SequenceValue::Structure(s) => {
                Some(VariableValue::Structure(s.evaluate_at_compiletime(params, context)))
            }
            SequenceValue::Operation(op) => {
                op.get()
                    .instantiate(params.clone())
                    .process_at_compiletime(context)
            }
        }
    }

    pub fn evaluate_at_runtime(&self, params: &Vec<Expression>, context: &mut context::Runtime) -> Option<VariableValue> {
        match self {
            SequenceValue::Structure(s) => {
                Some(VariableValue::Structure(s.evaluate_at_runtime(params, context)))
            }
            SequenceValue::Operation(op) => {
                op.get()
                    .instantiate(params.clone())
                    .process(context)
            }
        }
    }

    pub fn get_top_level_operation(&self) -> Option<TopLevelOperation> {
        if let SequenceValue::Operation(OperationTemplateEnum::TopLevel(op)) = self {
            Some(*op)
        } else {
            None
        }
    }

    pub fn is_operation(&self) -> bool {
        matches!(self, Self::Operation(_))
    }
}

/// Sequence is intuitively a sequence of words.
/// It corresponds to whole signatures, such as `move Pos by Pos`.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Sequence (Vec<Word>,Location);

impl Sequence {
    pub fn new(location: Location) -> Self {
        Self(vec![], location)
    }

    pub fn from(v: Vec<Word>, location: Location) -> Self {
        Self(v, location)
    }

    /// Get the underlying vector of words.
    pub fn get(&self) -> &Vec<Word> {
        &self.0
    }

    /// Get the source code location of this sequence.
    pub fn get_location(&self) -> &Location {
        &self.1
    }

    pub fn into_vec(self) -> Vec<Word> {
        self.0
    }

    /// Get word at given index
    pub fn at(&self, index: usize) -> &Word {
        &self.0[index]
    }

    /// Number of words in this sequence.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Append another word to the end of this sequence.
    pub fn push(&mut self, word: Word) {
        self.0.push(word);
    }

    /// Get all occuring types in the sequence (in the same order).
    pub fn get_types(&self) -> Vec<&VariableType> {
        let mut ret = vec![];
        for w in &self.0 {
            if let Some(t) = w.get_type() {
                ret.push(t);
            }
        }
        ret
    }

    pub fn compute_return_type(&self, ret: &VariableType, params: &Vec<Variable>) -> VariableType {
        if let Some(binding) = ret.get_binding() {
            // find a parameter that has this binding
            // take the type from passed params
            let pos = self.get_types().iter().position(|t| t.get_binding() == Some(binding)).unwrap();
            let type_depth = self.get_types()[pos].get_depth();
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
            param_type.wrap_depth(ret.get_depth());
            param_type
        } else {
            ret.clone()
        }
    }

    pub fn ___compute_return_type(&self, ret: &VariableType, params: &Vec<VariableType>) -> VariableType {
        if let Some(binding) = ret.get_binding() {
            // find a parameter that has this binding
            // take the type from passed params
            let pos = self.get_types().iter().position(|t| t.get_binding() == Some(binding)).unwrap();
            let type_depth = self.get_types()[pos].get_depth();
            let mut param_type = params[pos].clone();
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
            param_type.wrap_depth(ret.get_depth());
            param_type
        } else {
            ret.clone()
        }
    }

    /// Get all occuring types in the sequence (in the same order).
    pub fn get_types_cloned(&self) -> Vec<VariableType> {
        let mut ret = vec![];
        for w in &self.0 {
            if let Some(t) = w.get_type() {
                ret.push(t.clone());
            }
        }
        ret
    }

    /// Swap types of this sequence.
    /// TODO: there must be enough types. If there is more, it is negledged (probably members)
    pub fn swap_types(&mut self, types: &Vec<VariableType>) {
        let s_len = self.get_types().len();
        let t_len = types.len();
        assert!(s_len <= t_len, "error: expected at least {s_len} types, got {t_len}");

        let mut i = 0;
        for w in &mut self.0 {
            if !w.is_type() { continue; }

            let t = types[i].clone();
            i += 1;
            *w = Word::Type(t);
        }
    }

    /// Swap the nth type (specified by `at`) with a new one (`t`).
    pub fn swap_type_at(&mut self, at: usize, t: VariableType) {
        let mut i = 0;
        for w in &mut self.0 {
            if !w.is_type() { continue; }

            if i != at {
                i += 1;
                continue;
            }

            *w = Word::Type(t);
            return;
        }
    }
}

impl Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        for i in 0..self.0.len()-1 {
            write!(f, "{} ", self.0[i].to_string())?
        }
        write!(f, "{}", self.0[self.0.len()-1].to_string())
    }
}

#[macro_export]
macro_rules! seq {
    ( $($x:tt)+ ) => {
        Sequence::from(([$(word!($x)),+]).to_vec(), crate::translator::error::Location::default())
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{word,vtype};

    // TODO: not necessary but nice to have
    // #[test]
    // fn test_macro() {
    // }

    #[test]
    fn test_get_types() {
        let s = seq!("..." Int "..." "..." Pos "..." (Any(0)) "..." [String]);
        let types = s.get_types();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0], &vtype!(Int));
        assert_eq!(types[1], &vtype!(Pos));
        assert_eq!(types[2], &vtype!(Any(0)));
        assert_eq!(types[3], &vtype!([String]));
    }

    #[test]
    fn test_swap_types() {
        let mut s = seq!("..." Pos "..." "..." String "..." Color "..." (Any(0)));
        let new_types = vec![vtype!(Int), vtype!(Pos), vtype!(Any(0)), vtype!([String])];
        s.swap_types(&new_types);

        let types = s.get_types();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0], &vtype!(Int));
        assert_eq!(types[1], &vtype!(Pos));
        assert_eq!(types[2], &vtype!(Any(0)));
        assert_eq!(types[3], &vtype!([String]));
    }

    #[test]
    fn test_swap_type_at() {
        let mut s = seq!("..." Pos "..." "..." String "..." Color "..." (Any(0)));
        s.swap_type_at(0, vtype!(Int));
        s.swap_type_at(1, vtype!(Pos));
        s.swap_type_at(2, vtype!(Any(0)));
        s.swap_type_at(3, vtype!([String]));

        let types = s.get_types();
        assert_eq!(types.len(), 4);
        assert_eq!(types[0], &vtype!(Int));
        assert_eq!(types[1], &vtype!(Pos));
        assert_eq!(types[2], &vtype!(Any(0)));
        assert_eq!(types[3], &vtype!([String]));
    }
}
