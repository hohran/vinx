use std::collections::HashMap;

use crate::{ event::Operations, translator::{sequence::{Sequence, SequenceType}, type_constraints::TypeConstraints}, variable::VariableType};

use super::{State, StateId, super::{SequenceValue, Word}};

/// Structure for registering sequences, determining their values (method `run`), and infering types
/// or getting interpretations of ambiguous ones (method `get_interpretations`).
/// It is called automaton because of its underlying functionality derived from 
/// finite state automata.
pub struct Automaton {
    states: Vec<State>,
    return_values: HashMap<StateId, SequenceValue>,
    loaded: HashMap<SequenceType, usize>,
}

impl Automaton {
    pub fn new() -> Self {
        let loaded = HashMap::from([(SequenceType::Operation, 0), (SequenceType::Structure, 0)]);
        Self { states: vec![State::new()], return_values: HashMap::new(), loaded }
    }

    /// Creates a new state without any transitions and returns its id.
    pub fn new_state(&mut self) -> StateId {
        let new_state_id = self.states.len();
        self.states.push(State::new());
        new_state_id
    }

    /// Registers a return value for given sequence.
    /// This function will return `false`, if such sequence is already registered.
    /// _Theoretically speaking, this operation is equivalent to an automata union._
    pub fn register(&mut self, seq: Sequence, seq_type: SequenceType) -> bool {
        let mut cur_state = 0;
        for transition in seq.into_vec() {
            let next_state = self.states[cur_state].use_transition(&transition);
            if let Some(n) = next_state {
                cur_state = n;
                continue;
            }
            // if current state does not have this transition
            let new_state = self.new_state();
            self.states[cur_state].add_transition(transition, new_state);
            cur_state = new_state;
        }
        if self.return_values.contains_key(&cur_state) {
            return false;
        }
        let seq_id = self.loaded.get_mut(&seq_type).expect("error: sequence type without set count");
        self.return_values.insert(cur_state, seq_type.to_value(*seq_id));
        *seq_id += 1;
        true
    }

    /// Performs a run of automaton over sequence `seq` and returns its value (if it exists).
    pub fn run(&self, seq: &Vec<Word>) -> Option<SequenceValue> {
        // if seq.len() == 1 && let Some(t) = seq[0].get_type() {
        //     return Some(SequenceValue::Value(t.clone()));
        // }
        self.run_from(seq, 0, &TypeConstraints::new())
    }

    /// Performs a run of automaton over the rest of sequence `seq` from state `cur`.
    fn run_from(&self, seq: &[Word], cur: StateId, binding: &TypeConstraints) -> Option<SequenceValue> {
        if seq.is_empty() {
            return self.return_values.get(&cur).cloned();
        }
        let w = &seq[0];
        let rest = &seq[1..];
        if w.is_type() {
            let ts = self.states[cur].get_ordered_transitions(w, binding);
            for t in ts {
                if t.is_ambiguous() {
                    let mut binding_clone = binding.clone();
                    if let Some(next_state) = self.states[cur].apply(t.get(), &mut binding_clone) {
                        if let Some(s) = self.run_from(rest, next_state, &binding_clone) {
                            return Some(s);
                        }
                    }
                } else {
                    let next = self.states[cur].use_transition(t.get()).unwrap();
                    if let Some(sv) = self.run_from(rest, next, binding) {
                        return Some(sv);
                    }
                }
            }
        } else if let Some(next) = self.states[cur].use_transition(w) {
            return self.run_from(rest, next, binding);
        }
        return None;
    }

    /// Get all possible interpretations of sequence `seq`.
    /// If this sequence is assigned to a variable, `ret_id` should be set to its id.
    pub fn get_interpretations(&self, seq: &Vec<Word>, ret_var: Option<&VariableType>, operations: &Operations) -> Vec<TypeConstraints> {
        if seq.len() == 1 && let Word::Type(t) = &seq[0] {
            if let Some(var_type) = ret_var {
                // TODO: check if we shouldnt switch the intersection
                let Some(prod) = var_type.intersect(t) else {
                    return vec![];
                };
                let mut tc = TypeConstraints::new();
                if let Some(var_binding) = var_type.get_binding() {
                    tc.intersect_var(var_binding, &prod);
                }
                return vec![tc];
            } else {
                return vec![TypeConstraints::new()];
            }
        }
        self.get_interpretations_from(seq, 0, TypeConstraints::new(), TypeConstraints::new(), ret_var, operations)
    }

    fn get_interpretations_from(&self, seq: &[Word], cur: StateId, mut cur_constraints: TypeConstraints, mut binding: TypeConstraints, ret_var: Option<&VariableType>, operations: &Operations) -> Vec<TypeConstraints> {
        if seq.is_empty() {
            let Some(sv) = self.return_values.get(&cur) else { return vec![] };
            if let Some(var_type) = ret_var {
                let return_type = sv.into_type(operations).with_inverted_binding();
                // TODO: check if we shouldnt switch the intersection
                let Some(prod) = var_type.intersect(&return_type) else {
                    return vec![];
                };
                if let Some(var_binding) = var_type.get_binding() {
                    cur_constraints.intersect_var(var_binding, &prod);
                }
            }
            cur_constraints.refresh_bindings();
            return vec![cur_constraints];
        }
        let w = &seq[0];
        let rest = &seq[1..];
        if w.is_ambiguous() {
            let mut out = vec![];
            for t in self.states[cur].get_type_transitions() {
                let binding_clone = binding.clone();
                let mut constraint_clone = cur_constraints.clone();
                if let Some(n) = self.states[cur].use_transition(t.get()) {
                    if let Some(var) = w.get_binding() {
                        let var_depth = w.get_type().unwrap().get_depth();
                        if !constraint_clone.intersect_var(var, &t.get_type().unwrap_depth(var_depth).with_inverted_binding()) {
                            continue;
                        }
                    }
                    out.append(&mut self.get_interpretations_from(rest, n, constraint_clone, binding_clone, ret_var, operations));
                }
            }
            return out;
        } else {
            if let Some(next) = self.states[cur].apply(w, &mut binding) {
                return self.get_interpretations_from(rest, next, cur_constraints, binding, ret_var, operations);
            } else {
                return vec![];
            }
        }
    }

    /// Returns all sequences in the given automaton.
    #[allow(dead_code)]
    pub fn get_all_sequences(&self) -> Vec<(Sequence,SequenceValue)> {
        self.get_all_sequences_rec(0, &vec![])
    }

    #[allow(dead_code)]
    fn get_all_sequences_rec(&self, start: StateId, seq: &Vec<Word>) -> Vec<(Sequence,SequenceValue)> {
        let mut ret = vec![];
        if let Some(r) = self.return_values.get(&start) {
            ret.push((Sequence::from(seq.clone()),r.clone()));
        }
        for (t,new_state) in self.states[start].get_all_transitions() {
            let mut new_seq = seq.clone();
            new_seq.push(t.get().clone());
            ret.append(&mut self.get_all_sequences_rec(*new_state, &new_seq));
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{OperationTemplate, OperationTemplateEnum};
    use crate::event::builtins::Builtin;
    use crate::variable::VariableType;

    use super::{Automaton};
    use super::Word;
    use super::SequenceValue;
    use crate::translator::sequence::{Sequence, SequenceType};
    use crate::{seq,word,vtype};
    use crate::event::builtins::*;

    macro_rules! builtin {
        (($($w:tt)+), $f:expr, $r:expr) => {
            ((seq!($($w)+)), Some($r), $f)
        };

        (($($w:tt)+), $f:expr) => {
            ((seq!($($w)+)), None, $f)
        };
    }

    macro_rules! builtins {
        (
            $(
                ($($w:tt)+) $(=> $r:expr)?, $f:expr
            );* $(;)?
        ) => {
            [
                $(
                    builtin!(($($w)+), $f $(, $r)?)
                ),*
            ]
        };
    }

    fn load_operations(builtins: &[(Sequence, Option<VariableType>, Builtin)]) -> (Automaton, Vec<OperationTemplateEnum>) {
        let mut aut = Automaton::new();
        let mut ops = vec![];
        for (i,(seq,ret,op)) in builtins.into_iter().enumerate() {
            if !aut.register(seq.clone(), SequenceType::Operation) {
                panic!("error: union did not create any new states");
            }
            ops.push(OperationTemplateEnum::Standard(OperationTemplate::from_builtin(i, seq.clone(), *op, ret.clone())));
        }
        (aut,ops)
    }

    impl Automaton {
        pub fn from(la: (Sequence,SequenceType)) -> Self {
            let mut s = Self::new();
            s.register(la.0, la.1);
            s
        }
        pub fn len(&self) -> usize {
            self.states.len()
        }
    }

    #[test]
    fn test_automaton_run() {
        let la = (seq!("a" [Int] "b"), SequenceType::Operation);
        let a = Automaton::from(la);
        assert_eq!(a.len(), 4);
        assert_eq!(a.run(seq!("a" [Int] "b").get()), Some(SequenceValue::Operation(0)));
        assert_eq!(a.run(seq!("a" [Pos] "b").get()), None);
        assert_eq!(a.run(seq!("a" [Int] "c").get()), None);
    }

    #[test]
    fn test_automaton_union() {
        let la1 = (seq!("a" [Int] "b"), SequenceType::Operation);
        let la2 = (seq!("a" Pos "c"), SequenceType::Operation);
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.register(la2.0,la2.1);
        assert_eq!(a.len(), 6);
        assert_eq!(a.run(seq!("a" [Int] "b").get()), Some(SequenceValue::Operation(0)));
        assert_eq!(a.run(seq!("a" Pos "c").get()), Some(SequenceValue::Operation(1)));
        assert_eq!(a.run(seq!("a" Pos "b").get()), None);
    }

    #[test]
    fn test_automaton_priority_choice() {
        let la1 = (seq!(a Int b), SequenceType::Operation);
        let la2 = (seq!(a (Any(0)) b), SequenceType::Operation);
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.register(la2.0,la2.1);
        assert_eq!(a.len(), 6);
        assert_eq!(
            a.run(seq!(a Int b).get()),
            Some(SequenceValue::Operation(0))
        );
    }

    #[test]
    fn test_automaton_priority_choice_vec() {
        let la1 = (seq!("a" [Int] "b"), SequenceType::Operation);
        let la2 = (seq!("a" (Any(0)) "b"), SequenceType::Operation);
        let la3 = (seq!("a" [Any(0)] "b"), SequenceType::Operation);
        let mut a = Automaton::from(la1);
        assert_eq!(a.len(), 4);
        a.register(la2.0,la2.1);
        assert_eq!(a.len(), 6);
        a.register(la3.0,la3.1);
        assert_eq!(a.len(), 8);
        assert_eq!(a.run(seq!("a" [Int] "b").get()), Some(SequenceValue::Operation(0)));
        assert_eq!(a.run(seq!("a" Int "b").get()), Some(SequenceValue::Operation(1)));
        assert_eq!(a.run(seq!("a" [Pos] "b").get()), Some(SequenceValue::Operation(2)));
    }

    #[test]
    fn test_automaton_backtrace_small() {
        let la1 = (seq!(a Int b), SequenceType::Operation);
        let la2 = (seq!(a (Any(0)) c), SequenceType::Operation);
        let mut a = Automaton::from(la1);
        a.register(la2.0,la2.1);
        assert_eq!(a.run(seq!(a Int c).get()), Some(SequenceValue::Operation(1)));
    }

    #[test]
    fn test_automaton_backtrace_large() {
        let la1 = (seq!("a" Int "a" Int "a"), SequenceType::Operation);
        let la2 = (seq!("a" (Any(0)) "a" Pos "a"), SequenceType::Operation);
        let la3 = (seq!("a" (Any(0)) "a" Int "a"), SequenceType::Operation);
        let la4 = (seq!("a" Int "a" (Any(0)) "b"), SequenceType::Operation);
        let la5 = (seq!("a" Int "a" Pos "b"), SequenceType::Operation);
        let mut a = Automaton::from(la1);
        a.register(la2.0,la2.1);
        a.register(la3.0,la3.1);
        a.register(la4.0,la4.1);
        a.register(la5.0,la5.1);
        assert_eq!(a.run(seq!("a" Int "a" Int "a").get()), Some(SequenceValue::Operation(0)));
        assert_eq!(a.run(seq!("a" Int "a" Pos "a").get()), Some(SequenceValue::Operation(1)));
        assert_eq!(a.run(seq!("a" Color "a" Int "a").get()), Some(SequenceValue::Operation(2)));
        assert_eq!(a.run(seq!("a" Int "a" Int "b").get()), Some(SequenceValue::Operation(3)));
        assert_eq!(a.run(seq!("a" Int "a" Pos "b").get()), Some(SequenceValue::Operation(4)));
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
            let la = (ops[i].clone(), SequenceType::Operation);
            a.register(la.0,la.1);
        }
        let paths = a.get_interpretations(seq!("a" (Any(0)) "b").get(), None, &vec![]);
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
            let la = (ops[i].clone(), SequenceType::Operation);
            a.register(la.0,la.1);
        }
        let paths = a.get_interpretations(seq!(a (Any(0)) b (Any(0)) c).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        // 3)
        let ops = [
            seq!(a [Int] b),
            seq!(a [Pos] b),
            seq!(a [Any(0)] b),
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let la = (ops[i].clone(), SequenceType::Operation);
            a.register(la.0,la.1);
        }
        let paths = a.get_interpretations(seq!(a (Any(0)) b).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(seq!(a [Any(0)] b).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(seq!(a [Int] b).get(), None, &vec![]);
        assert_eq!(paths.len(), 1);
        // 4)
        let ops = [
            seq!(a [Any(0)] (Any(0))),
            seq!(a [Any(0)] Int),
            seq!(a [Any(0)] Pos),
        ];
        let mut a = Automaton::new();
        for i in 0..ops.len() {
            let la = (ops[i].clone(), SequenceType::Operation);
            a.register(la.0,la.1);
        }
        let paths = a.get_interpretations(seq!(a [Any(0)] (Any(0))).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(seq!(a (Any(0)) (Any(0))).get(), None, &vec![]);
        assert_eq!(paths.len(), 0);
        let paths = a.get_interpretations(seq!(a [Int] Color).get(), None, &vec![]);
        assert_eq!(paths, vec![]);
        // 5)
        let builtins: &[(Sequence, Option<VariableType>, Builtin)] = &builtins!(
            ("a" Int) => VariableType::Int, add_to;
            ("a" [Int]) => VariableType::Int, add_to;
            ("a" Pos) => VariableType::Pos, add_to;
            );
        let (aut, ops) = load_operations(builtins);
        let seq = seq!(a (Any(0)));
        assert_eq!(aut.get_interpretations(seq.get(), None, &ops).len(), 3);
        assert_eq!(aut.get_interpretations(seq.get(), Some(&VariableType::Any(1)), &ops).len(), 3);
        assert_eq!(aut.get_interpretations(seq.get(), Some(&VariableType::Int), &ops).len(), 2);
    }

    #[test]
    fn test_run() {
        let la = (seq!("move" Pos Direction "by" Int), SequenceType::Operation);
        let mut a = Automaton::from(la);
        a.register(seq!("set" (Any(0)) "to" (Any(0))), SequenceType::Operation);
        let s = seq!("move" Pos Direction "by" Int);
        let x = a.run(s.get());
        assert_eq!(x.unwrap(), SequenceValue::Operation(0));
        let x = a.run(seq!("set" Int "to" Int).get());
        assert_eq!(x.unwrap(), SequenceValue::Operation(1));
    }
}
