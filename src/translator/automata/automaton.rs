use std::collections::HashMap;

use crate::{ translator::{Sequence, automata::state::StateId, type_constraints::TypeConstraints}, variable::types::VariableType};

use super::{super::{SequenceValue, Word}, linear_automaton::LinearAutomaton, state::State};

pub struct Automaton {
    states: Vec<State>,
    return_values: HashMap<StateId, SequenceValue>,
}

impl Automaton {
    pub fn new() -> Self {
        Self { states: vec![State::new()], return_values: HashMap::new() }
    }

    /// Creates a new state without any transitions and returns its id.
    pub fn new_state(&mut self) -> StateId {
        let new_state_id = self.states.len();
        self.states.push(State::new());
        new_state_id
    }

    /// Performs a union with a linear automaton `la`.
    /// Returns `true` if the union changed the automaton (it created a new state, or set a return
    /// value on already existing state), and `false` otherwise.
    /// This method will panic if a return value is tried to be changed.
    pub fn union(&mut self, la: LinearAutomaton) -> bool {
        let mut branched = false;
        let mut cur_state = 0;
        for transition in la.transitions {
            let next_state = self.states[cur_state].get_exact_transition(&transition);
            if let Some(n) = next_state {
                cur_state = n;
                continue;
            }
            // if current state does not have this transition
            branched = true;
            let new_state = self.new_state();
            self.states[cur_state].add_transition(transition, new_state);
            cur_state = new_state;
        }
        // after iteration
        if branched {
            self.return_values.insert(cur_state, la.return_value);
            return true;
        }
        if self.return_values.get(&cur_state).is_some() {
            return false;
        }
        self.return_values.insert(cur_state, la.return_value);
        true
    }

    /// Performs a run of automaton over the sequence `seq`.
    /// Returns a value of this sequence if any.
    pub fn run(&self, seq: &Vec<Word>) -> Option<&SequenceValue> {
        self.run_from(seq, 0, &HashMap::new())
    }

    /// Performs a run of automaton over the rest of sequence `seq` from state `start`.
    fn run_from(&self, seq: &[Word], start: StateId, bind_mapping: &HashMap<usize,VariableType>) -> Option<&SequenceValue> {
        if seq.len() == 0 {
            return self.return_values.get(&start);
        }
        if let Word::Type(_) = &seq[0] {
            let ts = self.states[start].get_possible_transitions(&seq[0], bind_mapping);
            for t in ts {
                if t.is_ambiguous() {
                    let mut bind_mapping_clone = bind_mapping.clone();
                    if let Some(new_start) = self.states[start].get_transition(t, &mut bind_mapping_clone) {
                        if let Some(s) = self.run_from(&seq[1..], new_start, &bind_mapping_clone) {
                            return Some(s);
                        }
                    }
                } else {
                    let new_start = self.states[start].get_exact_transition(t).unwrap();
                    if let Some(s) = self.run_from(&seq[1..], new_start, bind_mapping) {
                        return Some(s);
                    }
                }
            }
        } else if let Some(n) = self.states[start].get_exact_transition(&seq[0]) {
            return self.run_from(&seq[1..], n, bind_mapping);
        }
        return None;
    }

    // pub fn refine_type_constraints(&self, seq: &Sequence, type_constraints: &TypeConstraints) -> Vec<TypeConstraints> {
    //     self.refine_type_constraints_from(seq, type_constraints, 0, &mut HashMap::new())
    // }
    //
    // pub fn refine_type_constraints_from(&self, seq: &[Word], current_type_constraints: &TypeConstraints, start: StateId, bind_mapping: &mut HashMap<usize,VariableType>) -> Vec<TypeConstraints> {
    //     if seq.len() == 0 {
    //         let mut constraint_clone = current_type_constraints.clone();
    //         constraint_clone.refresh_bindings();
    //         return vec![constraint_clone];
    //     }
    //     let mut out = vec![];
    //     let w = &seq[0];
    //     if w.is_ambiguous() {
    //         let branches = self.states[start].get_type_transitions();
    //         for branch_word in branches {
    //             let mut bind_mapping_clone = bind_mapping.clone();
    //             let mut constraint_clone = current_type_constraints.clone();
    //             if let Some(n) = self.states[start].get_transition(branch_word, &mut bind_mapping_clone) { // we expect only ambi types of Any
    //                 if let Some(binding) = w.get_binding() {
    //                     let var_type = w.get_variable_type().unwrap();
    //                     if let Some(intersected_type) = var_type.intersect(&branch_word.get_variable_type().unwrap().with_inverted_binding()) {
    //                         constraint_clone.update_binding(binding, intersected_type);
    //                         out.append(&mut self.refine_type_constraints_from(&seq[1..], &constraint_clone, n, bind_mapping));
    //                     } else {
    //                         panic!("error: somehow inapplicable transition, idk");
    //                         // return vec![];
    //                     }
    //                 }
    //             }
    //         }
    //     } else { // not ambiguous
    //         if let Some(n) = self.states[start].get_transition(w, bind_mapping) {
    //             return self.refine_type_constraints_from(&seq[1..], current_type_constraints, n, bind_mapping);
    //         } else {
    //             return out;
    //         }
    //     }
    //     out
    // }

    pub fn get_interpretations(&self, seq: &Vec<Word>, var_count: usize) -> Vec<TypeConstraints> {
        self.get_interpretations_from(seq, 0, TypeConstraints::new(var_count), &mut HashMap::new())
    }

    fn get_interpretations_from(&self, seq: &[Word], start: StateId, mut current_type_constraints: TypeConstraints, bind_mapping: &mut HashMap<usize,VariableType>) -> Vec<TypeConstraints> {
        if seq.len() == 0 {
            current_type_constraints.refresh_bindings();
            return vec![current_type_constraints];
        }
        let mut out = vec![];
        let w = &seq[0];
        if w.is_ambiguous() {
            let branches = self.states[start].get_type_transitions();
            for branch_word in branches {
                let mut bind_mapping_clone = bind_mapping.clone();
                let mut constraint_clone = current_type_constraints.clone();
                if let Some(n) = self.states[start].get_exact_transition(branch_word) {
                    if branch_word.is_ambiguous() {
                    }
                    if let Some(var_id) = w.get_binding() {
                        let var_depth = w.get_variable_type().unwrap().get_depth();
                        if !constraint_clone.intersect_var(&branch_word.get_variable_type().unwrap().unwrap_depth(var_depth).with_inverted_binding(), var_id) {
                            continue;
                        }
                    }
                    out.append(&mut self.get_interpretations_from(&seq[1..], n, constraint_clone, &mut bind_mapping_clone));
                }
            }
            return out;
        } else {
            if let Some(n) = self.states[start].get_transition(w, bind_mapping) {
                return self.get_interpretations_from(&seq[1..], n, current_type_constraints, bind_mapping);
            } else {
                return vec![];
            }
        }
    }

    /// Returns all sequences in a given automaton in form: (sequence, return_type).
    #[allow(dead_code)]
    pub fn get_all_sequences(&self) -> Vec<(Sequence,SequenceValue)> {
        self.get_all_sequences_rec(0, &vec![])
    }

    #[allow(dead_code)]
    fn get_all_sequences_rec(&self, start: StateId, seq: &Vec<Word>) -> Vec<(Sequence,SequenceValue)> {
        let mut ret = vec![];
        if let Some(r) = self.return_values.get(&start) {
            ret.push((seq.clone(),r.clone()));
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
    use crate::translator::type_constraints::TypeConstraints;
    use crate::variable::types::VariableType;
    use crate::{seq,word,vtype};

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
            return_values.insert(states.len()-1, la.return_value);
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
        let la = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(1));
        assert_eq!(la.transitions.len(), 3);
        assert_eq!(la.return_value, SequenceValue::Operation(1));
    }

    #[test]
    fn test_automaton_run() {
        let la = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(1));
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
        let la1 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(1));
        let la2 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("c".to_string()),
        ], SequenceValue::Operation(2));
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.union(la2);
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
        let la1 = LinearAutomaton::new(seq!(a Int b), SequenceValue::Operation(1));
        let la2 = LinearAutomaton::new(seq!(a (Any(1)) b), SequenceValue::Operation(2));
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.union(la2);
        assert_eq!(a.len(), 6);
        assert_eq!(
            a.run(&seq!(a Int b)),
            Some(&SequenceValue::Operation(1))
            );
    }

    #[test]
    fn test_automaton_priority_choice_vec() {
        let la1 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Int))),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(1));
        let la2 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(2));
        let la3 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Any(1)))),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(3));
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.union(la2);
        assert_eq!(a.len(), 6);
        a.union(la3);
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
        let la1 = LinearAutomaton::new(seq!(a Int b), SequenceValue::Operation(1));
        let la2 = LinearAutomaton::new(seq!(a (Any(1)) c), SequenceValue::Operation(2));
        let mut a = Automaton::from(la1);
        a.union(la2);
        assert_eq!(a.run(&seq!(a Int c)), Some(&SequenceValue::Operation(2)));
    }

    #[test]
    fn test_automaton_backtrace_large() {
        let la1 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
        ], SequenceValue::Operation(1));
        let la2 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("a".to_string()),
        ], SequenceValue::Operation(2));
        let la3 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
        ], SequenceValue::Operation(3));
        let la4 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(4));
        let la5 = LinearAutomaton::new(vec![
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Int),
            Word::Keyword("a".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("b".to_string()),
        ], SequenceValue::Operation(5));
        let mut a = Automaton::from(la1);
        a.union(la2);
        a.union(la3);
        a.union(la4);
        a.union(la5);
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
    fn test_get_interpretations() {
        // 1)
        let ops = [
            seq!("a" (Any(0)) "b"),
            seq!("a" (Int) "b"),
            seq!("a" (Pos) "b"),
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let la = LinearAutomaton::new(ops[i].clone(), SequenceValue::Operation(i));
            a.union(la);
        }
        let paths = a.get_interpretations(&seq!("a" (Any(0)) "b"), 1);
        assert_eq!(paths.len(), 3);
        // 2)
        let ops = [
            seq!(a Int b Int c),
            seq!(a Int b (Any(1)) c),
            seq!(a Int b Pos c),
            seq!(a Pos b Int c),
            seq!(a Pos b Pos c),
            seq!(a (Any(1)) b (Any(1)) b),  // bad ending keyword
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let la = LinearAutomaton::new(ops[i].clone(), SequenceValue::Operation(i));
            a.union(la);
        }
        let paths = a.get_interpretations(&seq!(a (Any(0)) b (Any(0)) c), 2);
        assert_eq!(paths.len(), 3);
        // 3)
        let ops = [
            seq!(a [Int] b),
            seq!(a [Pos] b),
            seq!(a [Any(0)] b),
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let la = LinearAutomaton::new(ops[i].clone(), SequenceValue::Operation(i));
            a.union(la);
        }
        let paths = a.get_interpretations(&seq!(a (Any(0)) b), 2);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(&seq!(a [Any(0)] b), 2);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(&seq!(a [Int] b), 2);
        assert_eq!(paths.len(), 1);
        // 4)
        let ops = [
            seq!(a [Any(0)] (Any(0))),
            seq!(a [Any(0)] Int),
            seq!(a [Any(0)] Pos),
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let la = LinearAutomaton::new(ops[i].clone(), SequenceValue::Operation(i));
            a.union(la);
        }
        let paths = a.get_interpretations(&seq!(a [Any(0)] (Any(0))), 2);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(&seq!(a (Any(0)) (Any(0))), 2);
        assert_eq!(paths.len(), 0);
        let paths = a.get_interpretations(&seq!(a [Int] Color), 2);
        assert_eq!(paths.len(), 0);
    }

    #[test]
    // FIXME
    fn test_get_all_paths_bound() {
        let ops = [
            seq!(a (Any(1)) (Any(1)) Int Pos),  // bad ending
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let la = LinearAutomaton::new(ops[i].clone(), SequenceValue::Operation(i));
            a.union(la);
        }
        let paths = a.get_interpretations(&seq!(a (Any(0)) (Any(1)) (Any(1)) (Any(0))), 2);
        assert_eq!(paths, vec![]);
    }

    #[test]
    fn test_run() {
        let la = LinearAutomaton::new(seq!("move" Pos Direction "by" Int), SequenceValue::Operation(1));
        let a = Automaton::from(la);
        let s = seq!("move" Pos Direction "by" Int);
        let x = a.run(&s);
        assert_eq!(x.unwrap(), &SequenceValue::Operation(1));
    }
}
