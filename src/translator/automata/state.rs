use std::collections::HashMap;

use crate::{ translator::Word, variable::types::VariableType};

pub type StateId = usize;

/// State of a finite machine.
/// Each state is represented all possible transitions from it, and expects to be identified with a number.
///
/// Every transition is semi-deterministic, with the only non-deterministic one being for ambigous type Any.
/// Getting the next state is possible with `get_transition` which looks up the most fitting
/// transition, or if you want to force a certain transition, use `get_exact_transition`.
pub struct State {
    transitions: Vec<(Word,StateId)>,
}

impl State {
    /// Create new state with no transitions.
    pub fn new() -> Self {
        Self { transitions: vec![] }
    }

    /// Check if state has certain transition. The transition must match perfectly, even with
    /// bindings of ambiguous types.
    pub fn has_transition(&self, t: &Word) -> bool {
        self.transitions.iter().any(|(wt,_)| wt.strictly_matches(t))
    }
    
    /// Add a new transition to state `n`, using `t`.
    /// Prerequisity is that this transition does not exist.
    pub fn add_transition(&mut self, t: Word, n: StateId) {
        assert!(!self.has_transition(&t), "error: transition already exists");
        self.transitions.push((t,n));
    }

    /// Get the applicability of `s` on a transition over `t`.
    /// The higher the applicability, the more applicable this symbol is for the given transition.
    /// If `None` is returned, the symbol is not applicable.
    fn transition_applicability(t: &Word, s: &Word, bind_mapping: &HashMap<usize,VariableType>) -> Option<StateId> {
        return Self::_transition_applicability(t, s, bind_mapping, true);
    }

    fn _transition_applicability(t: &Word, s: &Word, bind_mapping: &HashMap<usize,VariableType>, unwind_any: bool) -> Option<StateId> {
        if t == s {
            return Some(usize::MAX);    // if s is Any, it will only map to any
        }
        match (t.clone(),s.clone()) {
            (Word::Type(x),Word::Type(y)) => {
                if let VariableType::Any(any_binding) = x {
                    if unwind_any == false {
                        return Some(0);
                    }
                    if let Some(t) = bind_mapping.get(&any_binding) {
                        if let Some(app) = Self::_transition_applicability(&Word::Type(t.clone()), s, bind_mapping, false) {
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
                        if let Some(app) = Self::_transition_applicability(&Word::Type(*xn.clone()), &Word::Type(*yn.clone()), bind_mapping, unwind_any) {
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
    
    /// Get state for transition `t`, if such transition exists.
    pub fn get_exact_transition(&self, t: &Word) -> Option<StateId> {
        for (wt,n) in &self.transitions {
            if wt.strictly_matches(t) {
                return Some(*n);
            }
        }
        None
    }

    // pub fn enforce_transition_with(&self, transition: &Word, s: &Word, bind_mapping: &mut HashMap<usize,VariableType>) -> Option<StateId> {
    //     let (w,n) = self.transitions.iter().find(|(w,_)| w == transition)?;
    //     match (transition,s) {
    //         (Word::Keyword(t),Word::Keyword(s)) => {
    //             if t == s {
    //                 Some(*n)
    //             } else {
    //                 None
    //             }
    //         }
    //         (Word::Type(t),Word::Type(s)) => {
    //             if t.get_depth() > s.get_depth() && !s.is_ambiguous() {
    //                 // e.g. [Pos] <- Int
    //                 return None;
    //             }
    //             if let Some(binding) = t.get_binding() {    // ambiguous transition
    //                 match bind_mapping.get(&binding) {
    //                     // TODO TODO TODO
    //                     Some(bind_type) => {
    //                     }
    //                     None => {
    //                     }
    //                 }
    //             } else {
    //                 if t == s {
    //                     Some(*n)
    //                 } else {
    //                     None
    //                 }
    //             }
    //         }
    //         _ => None
    //     }
    // }

    // /// Get state for transition `s`, if such transition exists, with using binding mapping.
    // pub fn get_exact_mapped_transition(&self, s: &Word, bind_mapping: &mut HashMap<usize,VariableType>) -> Option<StateId> {
    //     let Word::Type(t) = s else {
    //         return self.get_exact_transition(s);
    //     };
    //     for (w,n) in &self.transitions {
    //         let Word::Type(wt) = w else { continue };
    //         if t == wt {
    //             // check binding
    //             let Some(binding) = wt.get_binding() else {
    //                 return Some(*n);
    //             };
    //             match bind_mapping.get(&binding) {
    //                 Some(bt) => {
    //
    //                 }
    //                 None => {
    //                     // create this mapping
    //                     bind_mapping.insert(binding, wt.clone());
    //                     return Some(*n);
    //                 }
    //             }
    //         }
    //     }
    //     None
    // }

    /// Get the most fitting next state for symbol `t`.
    pub fn get_transition(&self, t: &Word, bind_mapping: &mut HashMap<usize,VariableType>) -> Option<StateId> {
        if let Word::Type(_) = t {
            self.get_transition_for_type(t, bind_mapping)
        } else {
            self.get_exact_transition(t)
        }
    }

    /// Get the most fitting next state for symbol `t`, which represents a variable type
    fn get_transition_for_type(&self, t: &Word, bind_mapping: &mut HashMap<usize,VariableType>) -> Option<StateId> {
        assert!(t.is_type());
        let mut best: Option<(StateId,usize,&Word)> = None;
        // get most fitting (best) transition
        for (wt,n) in &self.transitions {
            if let Some(app) = State::transition_applicability(wt,t, bind_mapping) {
                if let Some((_,b,_)) = best {
                    if app > b {
                        best = Some((*n,app,wt));
                    }
                } else {
                    best = Some((*n,app,wt));
                }
            }
        }
        // possibly update binding mapping
        if let Some((n,_,wt)) = best {
            let Some(wt) = wt.get_variable_type() else { panic!() };
            if wt.is_ambiguous() {
                let binding = wt.get_binding().unwrap();
                // update binding
                if bind_mapping.get(&binding).is_none() {
                    let depth = wt.get_depth();
                    let new_binding_value = t.get_variable_type().unwrap().unwrap_depth(depth);
                    bind_mapping.insert(binding,new_binding_value);
                }
            }
            return Some(n);
        }
        None
    }

    /// Get all transitions possible from this state.
    #[allow(dead_code)]
    pub fn get_all_transitions(&self) -> &Vec<(Word,StateId)>{
        &self.transitions
    }

    /// Get all transitions from this state, using the symbol `s`.
    /// Returned transitions are ordered from the most fitting to the least fitting.
    pub fn get_possible_transitions(&self, s: &Word, bind_mapping: &HashMap<usize,VariableType>) -> Vec<&Word> {
        let mut v = vec![];
        let mut apps = vec![];
        for (t,_) in &self.transitions {
            if let Some(a) = State::transition_applicability(t, s, bind_mapping) {
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

    /// Get all transitions over symbols representing variable types.
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
        let mut bind_mapping = HashMap::new();
        bind_mapping.insert(1, vtype!(Int));
        let pos_trans = s.get_possible_transitions(&word!(Int), &mut bind_mapping);
        assert_eq!(pos_trans, [&word!(Int),&word!(Any(1))]);
        bind_mapping.insert(1, vtype!(Pos));
        let pos_trans = s.get_possible_transitions(&word!(Int), &mut bind_mapping);
        assert_eq!(pos_trans, [&word!(Int)]);
    }
}

