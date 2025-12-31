// FIXME: currently, state cannot have two Any transitions
// this may be a problem because, we might want that one of the Any is bound and the other isnt
use std::collections::HashMap;

use crate::{ translator::{get_bounded_value, Word}, variable::types::VariableType};


pub struct State {
    transitions: Vec<(Word,usize)>,
}

impl State {
    pub fn new() -> Self {
        Self { transitions: vec![] }
    }

    pub fn has_transition(&self, t: &Word) -> bool {
        self.transitions.iter().any(|(wt,_)| wt == t)
    }
    
    pub fn add_transition(&mut self, t: Word, n: usize) {
        assert!(!self.has_transition(&t), "error: transition already exists");
        self.transitions.push((t,n));
    }

    fn transition_applicability(t: &Word, s: &Word, any_mapping: &HashMap<usize,VariableType>) -> Option<usize> {
        return Self::_transition_applicability(t, s, any_mapping, true);
    }

    fn _transition_applicability(t: &Word, s: &Word, any_mapping: &HashMap<usize,VariableType>, unwind_any: bool) -> Option<usize> {
        if t == s {
            return Some(usize::MAX);    // if s is Any, it will only map to any
        }
        match (t.clone(),s.clone()) {
            (Word::Type(x),Word::Type(y)) => {
                if let VariableType::Any(any_binding) = x {
                    if unwind_any == false {
                        return Some(0);
                    }
                    if let Some(t) = any_mapping.get(&any_binding) {
                        if let Some(app) = Self::_transition_applicability(&Word::Type(t.clone()), s, any_mapping, false) {
                            return Some(app.saturating_sub(1));
                        } else {
                            return None;
                        }
                    }
                    return Some(0);
                }
                match (&x,&y) {
                    (VariableType::Vec(xn),VariableType::Vec(yn)) => {
                        // TODO: think about this
                        if let Some(app) = Self::transition_applicability(&Word::Type(*xn.clone()), &Word::Type(*yn.clone()), any_mapping) {
                            if app < 10_000 {
                                return Some(app+2)
                            } else {
                                return Some(app)
                            }
                        }
                    }
                    _ => {return None;}
                }
                None
            },
            _ => None,
        }
    }
    
    pub fn get_exact_transition(&self, t: &Word) -> Option<usize> {
        for (wt,n) in &self.transitions {
            if wt == t {
                return Some(*n);
            }
        }
        None
    }

    pub fn get_transition(&self, t: &Word, any_mapping: &mut HashMap<usize,VariableType>) -> Option<usize> {
        if let Word::Type(_) = t {
            self.get_transition_for_type(t, any_mapping)
        } else {
            self.get_exact_transition(t)
        }
    }

    fn get_transition_for_type(&self, t: &Word, any_mapping: &mut HashMap<usize,VariableType>) -> Option<usize> {
        let mut best: Option<(usize,usize,&Word)> = None;
        for (wt,n) in &self.transitions {
            if let Some(app) = State::transition_applicability(wt,t, any_mapping) {
                if let Some((_,b,_)) = best {
                    if app > b {
                        best = Some((*n,app,wt));
                    }
                } else {
                    best = Some((*n,app,wt));
                }
            }
        }
        if let Some((n,_,wt)) = best {
            if wt.is_ambiguous() {
                let (binding,value) = get_bounded_value(wt, t);
                // update binding
                if any_mapping.get(&binding).is_none() {
                    any_mapping.insert(binding,Self::word_type(&value));
                }
            }
            return Some(n);
        }
        None
    }

    pub fn get_all_transitions(&self) -> &Vec<(Word,usize)>{
        &self.transitions
    }

    fn word_type(w: &Word) -> VariableType {
        if let Word::Type(t) = w {
            t.clone()
        } else {
            panic!("error: expected word to be type, got {:?}", w)
        }
    }

    pub fn get_possible_transitions(&self, s: &Word, any_mapping: &HashMap<usize,VariableType>) -> Vec<&Word> {
        let mut v = vec![];
        let mut apps = vec![];
        for (t,_) in &self.transitions {
            if let Some(a) = State::transition_applicability(t, s, any_mapping) {
                let mut i = 0;
                for _ in 0..apps.len() {
                    if a > apps[i] {
                        break;
                    }
                    i += 1;
                }
                apps.insert(i, a);
                v.insert(i, t);
            }
        }
        v
    }

    pub fn get_type_transitions(&self) -> Vec<&Word> {
        self.transitions.iter().filter(|(t,_)| if let Word::Type(_) = t { true } else { false }).map(|(t,_)| t).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{vtype, word};

    use super::*;

    #[test]
    fn test_possible_transition_ordering() {
        let mut s = State::new();
        s.add_transition(word!(Int), 1);
        s.add_transition(word!(Any(1)), 2);
        // s.add_transition(word!(Any(2)), 3);
        s.add_transition(word!(Pos), 4);
        let mut any_mapping = HashMap::new();
        any_mapping.insert(1, vtype!(Int));
        let pos_trans = s.get_possible_transitions(&word!(Int), &mut any_mapping);
        assert_eq!(pos_trans, [&word!(Int),&word!(Any(1))]);
        any_mapping.insert(1, vtype!(Pos));
        let pos_trans = s.get_possible_transitions(&word!(Int), &mut any_mapping);
        assert_eq!(pos_trans, [&word!(Int)]);
    }
}

