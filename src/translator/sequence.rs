use std::fmt::Display;

use crate::{translator::Word, variable::VariableType};

/// Sequence is intuitively a sequence of words.
/// It corresponds to whole signatures, such as `move Pos by Pos`.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Sequence (Vec<Word>);

impl Sequence {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn from(v: Vec<Word>) -> Self {
        Self(v)
    }

    /// Get the underlying vector of words.
    pub fn get(&self) -> &Vec<Word> {
        &self.0
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
        Sequence::from(([$(word!($x)),+]).to_vec())
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
