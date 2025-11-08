use crate::translator::{SequenceValue, Word};

pub struct LinearAutomaton {
    pub transitions: Vec<Word>,
    pub return_value: Option<SequenceValue>,
}

impl LinearAutomaton {
    // pub fn new() -> Self {
    //     Self { transitions: vec![], return_value: None }
    // }

    pub fn from(v: &Vec<Word>) -> Self {
        Self { transitions: v.clone(), return_value: None }
    }

    // pub fn add_state(&mut self, t: &Word) {
    //     self.transitions.push(t.clone());
    // }

    pub fn returns(&mut self, r: SequenceValue) {
        self.return_value = Some(r);
    }
}


