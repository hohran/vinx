use crate::translator::{SequenceValue, Word};

pub struct LinearAutomaton {
    pub transitions: Vec<Word>,
    pub return_value: SequenceValue,
}

impl LinearAutomaton {
    pub fn new(transitions: Vec<Word>, return_value: SequenceValue) -> Self {
        Self { transitions, return_value }
    }
}


