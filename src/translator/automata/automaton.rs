use std::collections::HashMap;


use crate::{ translator::type_constraints::TypeConstraints, variable::types::VariableType};

use super::{super::{SequenceValue, Word}, linear_automaton::LinearAutomaton, state::State};

pub struct Automaton {
    states: Vec<State>,
    return_values: HashMap<usize, SequenceValue>,
}

impl Automaton {
    pub fn new() -> Self {
        Self { states: vec![State::new()], return_values: HashMap::new() }
    }

    // pub fn from(la: LinearAutomaton) -> Self {
    //     let mut states = vec![State::new()];
    //     for t in &la.transitions {
    //         let state_count = states.len();
    //         let cur_state = state_count-1;
    //         states.push(State::new());
    //         states[cur_state].add_transition(t.clone(), state_count);
    //     }
    //     let mut return_values = HashMap::new();
    //     return_values.insert(states.len()-1, la.return_value.expect("error: linear automaton has unset return value"));
    //     Self { states, return_values }
    // }

    pub fn new_state(&mut self) -> usize {
        let state_count = self.states.len();
        self.states.push(State::new());
        state_count
    }

    pub fn union(&mut self, la: LinearAutomaton) -> Result<(),String> {
        let mut branched = false;
        let mut cur_state = 0;
        for i in 0..la.transitions.len() {
            let t = &la.transitions[i];
            if !branched && self.states[cur_state].has_transition(t) {
                // check that the next step is contained
                let next_state = self.states[cur_state].get_transition(t,&mut HashMap::new());
                if let Some(n) = next_state {
                    cur_state = n;
                    continue;
                }
                // otherwise branch
                branched = true;
            }
            // create a new state with this transition
            let new_state = self.new_state();
            self.states[cur_state].add_transition(t.clone(), new_state);
            cur_state = new_state;
        }
        if branched {
            self.return_values.insert(cur_state, la.return_value.expect("error: other automaton has unset return value"));
            return Ok(());
        }
        if self.return_values.get(&cur_state).is_some() {
            return Err("union did not create any new states".to_string());
        }
        self.return_values.insert(cur_state, la.return_value.expect("error: other automaton has unset return value"));
        Ok(())
    }

    pub fn run(&self, seq: &Vec<Word>) -> Option<&SequenceValue> {
        self.run_from(seq, 0, &HashMap::new())
    }

    fn run_from(&self, seq: &[Word], start: usize, any_mapping: &HashMap<usize,VariableType>) -> Option<&SequenceValue> {
        if seq.len() == 0 {
            return self.return_values.get(&start);
        }
        if let Word::Type(_) = &seq[0] {
            let ts = self.states[start].get_possible_transitions(&seq[0], any_mapping);
            for t in ts {
                if t.is_ambiguous() {
                    let mut any_mapping_clone = any_mapping.clone();
                    if let Some(new_start) = self.states[start].get_transition(t, &mut any_mapping_clone) {
                        if let Some(s) = self.run_from(&seq[1..], new_start, &any_mapping_clone) {
                            return Some(s);
                        }
                    }
                } else {
                    let new_start = self.states[start].get_exact_transition(t).unwrap();
                    if let Some(s) = self.run_from(&seq[1..], new_start, any_mapping) {
                        return Some(s);
                    }
                }
            }
        } else if let Some(n) = self.states[start].get_exact_transition(&seq[0]) {
            return self.run_from(&seq[1..], n, any_mapping);
        }
        return None;
    }

    // pub fn len(&self) -> usize {
    //     self.states.len()
    // }

    pub fn get_all_paths(&self, seq: &Vec<Word>, var_count: usize) -> Vec<TypeConstraints> {
        self.get_all_paths_from(seq, 0, TypeConstraints::new(var_count), &mut HashMap::new())
    }

    fn get_all_paths_from(&self, seq: &[Word], start: usize, mut current_type_constraints: TypeConstraints, any_mapping: &mut HashMap<usize,VariableType>) -> Vec<TypeConstraints> {
        if seq.len() == 0 {
            current_type_constraints.refresh_bindings();
            return vec![current_type_constraints];
        }
        let mut out = vec![];
        let w = &seq[0];
        if w.is_ambiguous() {
            assert!(matches!(w, Word::Type(VariableType::Any(_))), "error: expected Any type, got {w:?}");
            let branches = self.states[start].get_type_transitions();
            for branch_word in branches {
                let mut any_mapping_clone = any_mapping.clone();
                let mut constraint_clone = current_type_constraints.clone();
                if let Some(n) = self.states[start].get_transition(branch_word, &mut any_mapping_clone) { // we expect only ambi types of Any
                    if let Some(var_id) = w.get_binding() {
                        if !constraint_clone.intersect_var(&branch_word.get_variable_type().unwrap().with_inverted_binding(), var_id) {
                            return vec![];
                        }
                    }
                    out.append(&mut self.get_all_paths_from(&seq[1..], n, constraint_clone, &mut any_mapping_clone));
                }
            }
            return out;
        } else {
            if let Some(n) = self.states[start].get_transition(w, any_mapping) {
                return self.get_all_paths_from(&seq[1..], n, current_type_constraints, any_mapping);
            } else {
                return vec![];
            }
        }
    }

    // fn get_current_var_id(&self, seq: &[Word]) -> Option<usize> {
    //     if let Word::Type(VariableType::Any(var_id)) = &seq[0] {
    //         Some(*var_id)
    //     } else {
    //         None
    //     }
    // }

    pub fn get_all_sequences(&self) -> Vec<(SequenceValue,Vec<Word>)> {
        self.get_all_sequences_rec(0, &vec![])
    }

    fn get_all_sequences_rec(&self, start: usize, seq: &Vec<Word>) -> Vec<(SequenceValue,Vec<Word>)> {
        let mut ret = vec![];
        if let Some(r) = self.return_values.get(&start) {
            ret.push((r.clone(),seq.clone()));
        }
        for (w,new_state) in self.states[start].get_all_transitions() {
            let mut new_seq = seq.clone();
            new_seq.push(w.clone());
            ret.append(&mut self.get_all_sequences_rec(*new_state, &new_seq));
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{Automaton, LinearAutomaton, State};
    use super::Word;
    use super::SequenceValue;
    use crate::variable::types::VariableType;
    use crate::{seq,word};

    impl Automaton {
        pub fn from(la: LinearAutomaton) -> Self {
            let mut states = vec![State::new()];
            for t in &la.transitions {
                let state_count = states.len();
                let cur_state = state_count-1;
                states.push(State::new());
                states[cur_state].add_transition(t.clone(), state_count);
            }
            let mut return_values = HashMap::new();
            return_values.insert(states.len()-1, la.return_value.expect("error: linear automaton has unset return value"));
            Self { states, return_values }
        }
        pub fn len(&self) -> usize {
            self.states.len()
        }
    }

    #[test]
    fn test_box_behavior() {
        let vt1 = VariableType::Vec(Box::new(VariableType::Int));
        let vt2 = VariableType::Vec(Box::new(VariableType::Int));
        let vt3 = VariableType::Vec(Box::new(VariableType::Any(1)));
        let vt4 = VariableType::Vec(Box::new(VariableType::Vec(Box::new(VariableType::Int))));
        let vt4_ = VariableType::Vec(Box::new(VariableType::Vec(Box::new(VariableType::Int))));
        assert_eq!(vt1, vt2);
        assert_ne!(vt1,vt3);
        assert_ne!(vt1,vt4);
        let w1 = Word::Type(vt1);
        let w2 = Word::Type(vt2);
        let w3 = Word::Type(vt3);
        let w4 = Word::Type(vt4);
        let w4_ = Word::Type(vt4_);
        assert_eq!(w1, w2);
        assert_ne!(w1, w3);
        assert_ne!(w1, w4);
        assert_eq!(w4, w4_);
    }

    #[test]
    fn test_state_get_possible_transitions() {
        let mut s = State::new();
        s.add_transition(Word::Type(VariableType::Int), 1);
        s.add_transition(Word::Type(VariableType::Vec(Box::new(VariableType::Int))), 2);
        s.add_transition(Word::Type(VariableType::Vec(Box::new(VariableType::Any(1)))), 3);
        s.add_transition(Word::Type(VariableType::Any(1)), 4);
        let wti = Word::Type(VariableType::Int);
        let wtvi = Word::Type(VariableType::Vec(Box::new(VariableType::Int)));
        let wtva = Word::Type(VariableType::Vec(Box::new(VariableType::Any(1))));
        let wta = Word::Type(VariableType::Any(1));
        let ps_wti = s.get_possible_transitions(&wti,&mut HashMap::new());
        let ps_wtvi = s.get_possible_transitions(&wtvi,&mut HashMap::new());
        assert_eq!(ps_wtvi.len(), 3);
        assert_eq!(ps_wtvi, vec![&wtvi,&wtva,&wta]);
        assert_eq!(ps_wti.len(), 2);
        assert_eq!(ps_wti, vec![&wti,&wta]);
    }

    #[test]
    fn test_linear_automaton() {
        let mut la = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ]);
        la.returns(SequenceValue::Operation(1));
        assert_eq!(la.transitions.len(), 3);
        assert_eq!(la.return_value, Some(SequenceValue::Operation(1)));
    }

    #[test]
    fn test_automaton_run() {
        let mut la = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ]);
        la.returns(SequenceValue::Operation(1));
        let a = Automaton::from(la);
        assert_eq!(a.len(), 4);
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ]), Some(&SequenceValue::Operation(1)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Pos))),
            Word::Keyword("b".to_string()),
        ]), None);
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("c".to_string()),
        ]), None);
    }

    #[test]
    fn test_automaton_union() {
        let mut la1 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ]);
        la1.returns(SequenceValue::Operation(1));
        let mut la2 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("c".to_string()),
        ]);
        la2.returns(SequenceValue::Operation(2));
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.union(la2).unwrap();
        assert_eq!(a.len(), 6);
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ]), Some(&SequenceValue::Operation(1)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("c".to_string()),
        ]), Some(&SequenceValue::Operation(2)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("b".to_string()),
        ]), None);
    }

    #[test]
    fn test_automaton_priority_choice() {
        let mut la1 = LinearAutomaton::from(&seq!(a Int b));
        la1.returns(SequenceValue::Operation(1));
        let mut la2 = LinearAutomaton::from(&seq!(a (Any(1)) b));
        la2.returns(SequenceValue::Operation(2));
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.union(la2).unwrap();
        assert_eq!(a.len(), 6);
        assert_eq!(
            a.run(&seq!(a Int b)),
            Some(&SequenceValue::Operation(1))
            );
    }

    #[test]
    fn test_automaton_priority_choice_vec() {
        let mut la1 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ]);
        la1.returns(SequenceValue::Operation(1));
        let mut la2 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("b".to_string()),
        ]);
        la2.returns(SequenceValue::Operation(2));
        let mut la3 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Any(1)))),
            Word::Keyword("b".to_string()),
        ]);
        la3.returns(SequenceValue::Operation(3));
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.union(la2).unwrap();
        assert_eq!(a.len(), 6);
        a.union(la3).unwrap();
        assert_eq!(a.len(), 8);
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ]), Some(&SequenceValue::Operation(1)));
        assert_ne!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Pos))),
            Word::Keyword("c".to_string()),
        ]), Some(&SequenceValue::Operation(3)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("b".to_string()),
        ]), Some(&SequenceValue::Operation(2)));
    }

    #[test]
    fn test_automaton_backtrace_small() {
        let mut la1 = LinearAutomaton::from(&seq!(a Int b));
        la1.returns(SequenceValue::Operation(1));
        let mut la2 = LinearAutomaton::from(&seq!(a (Any(1)) c));
        la2.returns(SequenceValue::Operation(2));
        let mut a = Automaton::from(la1);
        a.union(la2).unwrap();
        assert_eq!(a.run(&seq!(a Int c)), Some(&SequenceValue::Operation(2)));
    }

    #[test]
    fn test_automaton_backtrace_large() {
        let mut la1 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
        ]);
        la1.returns(SequenceValue::Operation(1));
        let mut la2 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("a".to_string()),
        ]);
        la2.returns(SequenceValue::Operation(2));
        let mut la3 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
        ]);
        la3.returns(SequenceValue::Operation(3));
        let mut la4 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("b".to_string()),
        ]);
        la4.returns(SequenceValue::Operation(4));
        let mut la5 = LinearAutomaton::from(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("b".to_string()),
        ]);
        la5.returns(SequenceValue::Operation(5));
        let mut a = Automaton::from(la1);
        a.union(la2).unwrap();
        a.union(la3).unwrap();
        a.union(la4).unwrap();
        a.union(la5).unwrap();
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
        ]), Some(&SequenceValue::Operation(1)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("a".to_string()),
        ]), Some(&SequenceValue::Operation(2)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Color),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
        ]), Some(&SequenceValue::Operation(3)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("b".to_string()),
        ]), Some(&SequenceValue::Operation(4)));
        assert_eq!(a.run(&vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("b".to_string()),
        ]), Some(&SequenceValue::Operation(5)));
    }

    #[test]
    fn test_get_type_transitions() {
        let mut s = State::new();
        s.add_transition(Word::Keyword("a".to_string()), 1);
        s.add_transition(Word::Type(VariableType::Int), 2);
        s.add_transition(Word::Type(VariableType::Vec(Box::new(VariableType::Int))), 3);
        s.add_transition(Word::Keyword("b".to_string()), 4);
        let tts = s.get_type_transitions();
        assert_eq!(tts.len(), 2);
        assert_eq!(tts, vec![
            &Word::Type(VariableType::Int),
            &Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
        ]);
    }

    #[test]
    fn test_get_all_paths_count1() {
        let ops = [
            seq!("a" (Any(0)) "b"),
            seq!("a" (Int) "b"),
            seq!("a" (Pos) "b"),
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let mut la = LinearAutomaton::from(&ops[i]);
            la.returns(SequenceValue::Operation(i));
            a.union(la).unwrap();
        }

        let paths = a.get_all_paths(&seq!("a" (Any(0)) "b"), 1);
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_get_all_paths_count2() {
        let ops = [
            seq!(a Int b Int c),
            seq!(a (Int) b (Any(1)) c),
            seq!(a Int b Pos c),
            seq!(a Pos b Int c),
            seq!(a Pos b Pos c),
            seq!(a (Any(1)) b (Any(1)) b),  // bad ending keyword
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let mut la = LinearAutomaton::from(&ops[i]);
            la.returns(SequenceValue::Operation(i));
            a.union(la).unwrap();
        }
        let paths = a.get_all_paths(&seq!(a (Any(0)) b (Any(1)) c), 2);
        assert_eq!(paths.len(), 5);
    }

    #[test]
    fn test_get_all_paths_bound() {
        let ops = [
            seq!(a (Any(1)) (Any(1)) Int Pos),  // bad ending keyword
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let mut la = LinearAutomaton::from(&ops[i]);
            la.returns(SequenceValue::Operation(i));
            a.union(la).unwrap();
        }
        let paths = a.get_all_paths(&seq!(a (Any(0)) (Any(1)) (Any(1)) (Any(0))), 2);
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_run() {
        let mut la = LinearAutomaton::from(&seq!("move" Pos Direction "by" Int));
        la.returns(SequenceValue::Operation(1));
        let a = Automaton::from(la);
        let s = seq!("move" Pos Direction "by" Int);
        let x = a.run(&s);
        assert_eq!(x.unwrap(), &SequenceValue::Operation(1));
    }
}
