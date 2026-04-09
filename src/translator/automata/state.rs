use crate::translator::{Word, automata::transition::{Applicability, Transition}, type_constraints::TypeConstraints};

pub type StateId = usize;

/// State of a finite machine.
/// Each state is represented all possible transitions from it, and expects to be identified with a number.
///
/// Getting the next state is possible with `get_transition` which looks up the most fitting
/// transition, or if you want to force a certain transition, use `use_transition`.
pub struct State {
    transitions: Vec<(Transition,StateId)>,
}

impl State {
    /// Create new state with no transitions.
    pub fn new() -> Self {
        Self { transitions: vec![] }
    }

    /// Check if state has transition specified by `word`.
    pub fn has_transition(&self, word: &Word) -> bool {
        self.transitions.iter().any(|(t,_)| t.is(word))
    }
    
    /// Add a new transition to state `n`, using `t`.
    /// Prerequisity is that this transition does not exist.
    pub fn add_transition(&mut self, t: Word, s: StateId) {
        assert!(!self.has_transition(&t), "error: transition already exists");
        self.transitions.push((t.into(),s));
    }

    /// Get state for transition defined by `word`, if such transition exists.
    pub fn use_transition(&self, word: &Word) -> Option<StateId> {
        self.transitions.iter()
            .find(|(t,_)| t.is(word))
            .map(|(_,s)| *s)
    }

    /// Get the most fitting next state for word `word`, given binding for Any.
    pub fn apply(&self, word: &Word, binding: &mut TypeConstraints) -> Option<StateId> {
        if word.is_type() {
            self.get_transition_for_type(word, binding)
        } else {
            self.use_transition(word)
        }
    }

    /// Get the most fitting next state for a word, which represents variable type
    fn get_transition_for_type(&self, word: &Word, bind_mapping: &mut TypeConstraints) -> Option<StateId> {
        assert!(word.is_type());
        let mut best: Option<(StateId,Applicability,&Transition)> = None;
        // get most fitting (best) transition
        for (t,s) in &self.transitions {
            let Some(app) = t.get_applicability(word, &bind_mapping) else {
                continue;
            };
            if let Some((_,b,_)) = best {
                if app > b {
                    best = Some((*s,app,t));
                }
            } else {
                best = Some((*s,app,t));
            }
        }
        // possibly update binding mapping
        if let Some((s,_,t)) = best {
            let wt = t.get_type();
            if wt.is_ambiguous() {
                let binding = wt.get_binding().unwrap();
                // update binding
                let depth = wt.get_depth();
                let new_binding_value = word.get_type().unwrap().unwrap_depth(depth);
                bind_mapping.intersect_var(binding, new_binding_value);
            }
            return Some(s);
        }
        None
    }

    /// Get all transitions possible from this state.
    #[allow(dead_code)]
    pub fn get_all_transitions(&self) -> &Vec<(Transition,StateId)>{
        &self.transitions
    }


    /// Get all transitions from this state, using the word `word`.
    /// Returned transitions are ordered by their applicability.
    pub fn get_ordered_transitions(&self, s: &Word, bind_mapping: &TypeConstraints) -> Vec<&Transition> {
        let mut v = vec![];
        let mut apps = vec![];
        for (t,_) in &self.transitions {
            if let Some(a) = t.get_applicability(s, bind_mapping) {
                let mut i = 0;
                for a2 in &apps {
                    if a > *a2 { break; }
                    i += 1;
                }
                apps.insert(i, a);
                v.insert(i, t);
            }
        }
        v
    }

    /// Get all transitions over symbols representing variable types.
    pub fn get_type_transitions(&self) -> Vec<&Transition> {
        self.transitions.iter()
            .filter(|(t,_)| t.is_type())
            .map(|(t,_)| t)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{vtype, word};
    use crate::variable::VariableType;

    use super::*;

    #[test]
    fn test_get_ordered_transitions() {
        let mut s = State::new();
        s.add_transition(word!(Int), 1);
        s.add_transition(word!(Any(1)), 2);
        s.add_transition(word!(Pos), 4);
        let mut bind_mapping = TypeConstraints::_new();
        bind_mapping.intersect_var(1, &vtype!(Int));
        let pos_trans = s.get_ordered_transitions(&word!(Int), &bind_mapping);
        assert_eq!(pos_trans, [&Transition::from(word!(Int)),&Transition::from(word!(Any(1)))]);
        bind_mapping = TypeConstraints::_new();
        bind_mapping.intersect_var(1, &vtype!(Pos));
        let pos_trans = s.get_ordered_transitions(&word!(Int), &bind_mapping);
        assert_eq!(pos_trans, [&Transition::from(word!(Int))]);
    }
}
