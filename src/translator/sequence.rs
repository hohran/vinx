use std::fmt::Display;

use crate::{translator::Word, variable::VariableType};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Sequence (Vec<Word>);

impl Sequence {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn from(v: Vec<Word>) -> Self {
        Self(v)
    }

    pub fn get(&self) -> &Vec<Word> {
        &self.0
    }

    pub fn into_vec(self) -> Vec<Word> {
        self.0
    }

    pub fn at(&self, index: usize) -> &Word {
        &self.0[index]
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

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
}
