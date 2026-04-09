use std::fmt::Display;

use crate::{translator::Word, variable::VariableType};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Sequence (Vec<Word>);

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

    pub fn get_types(&self) -> Vec<&VariableType> {
        let mut ret = vec![];
        for w in &self.0 {
            if let Word::Type(t) = w {
                ret.push(t);
            }
        }
        ret
    }
}
