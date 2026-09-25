use std::collections::HashMap;

use crate::{ event::Operations, translator::{error::Location, sequence::Sequence, type_constraints::TypeConstraints}, variable::VariableType};

use super::{State, StateId, super::{SequenceValue, Word}};

/// Structure for registering sequences, determining their values (method `run`), and infering types
/// or getting interpretations of ambiguous ones (method `get_interpretations`).
/// It is called automaton because of its underlying functionality derived from 
/// finite state automata.
pub struct Automaton {
    states: Vec<State>,
    return_values: HashMap<StateId, SequenceValue>,
    loaded_operations: usize,
    loaded_structures: usize,
}

impl Automaton {
    pub fn new() -> Self {
        Self { states: vec![State::new()], return_values: HashMap::new(), loaded_operations: 0, loaded_structures: 0 }
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
    pub fn register(&mut self, seq: Sequence, seq_value: SequenceValue) -> bool {
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
        if seq_value.is_operation() {
            self.loaded_operations += 1;
        } else {
            self.loaded_structures += 1;
        }
        self.return_values.insert(cur_state, seq_value);
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
                let return_type = sv.get_general_return_type().with_inverted_binding();
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
    pub fn get_all_sequences(&self) -> Vec<(Sequence,SequenceValue)> {
        self.get_all_sequences_rec(0, &vec![])
    }

    fn get_all_sequences_rec(&self, start: StateId, seq: &Vec<Word>) -> Vec<(Sequence,SequenceValue)> {
        let mut ret = vec![];
        if let Some(r) = self.return_values.get(&start) {
            ret.push((Sequence::from(seq.clone(), Location::default()),r.clone()));
        }
        for (t,new_state) in self.states[start].get_all_transitions() {
            let mut new_seq = seq.clone();
            new_seq.push(t.get().clone());
            ret.append(&mut self.get_all_sequences_rec(*new_state, &new_seq));
        }
        ret
    }

    /// Returns the number of all registered sequences.
    pub fn get_count(&self) -> usize {
        self.return_values.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{Func, OperationTemplate};
    use crate::variable::{Variable, VariableType, VariableValue};
    use std::collections::HashMap;

    use super::Automaton;
    use super::Word;
    use super::SequenceValue;
    use crate::translator::sequence::Sequence;
    use crate::{context, seq, vtype, word};
    use crate::event::builtins::*;
    use general::Builtin;

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

    macro_rules! expect {
        (
            $(
                ($($w:tt)+) => $r:expr
            );* $(;)?
        ) => {{
            let mut map: HashMap<Sequence, Option<usize>> = HashMap::new();
            $( map.insert(seq!($($w)+), $r); )*
            map
        }}
    }

    // helper builtin function
    fn nop(_context: &mut dyn context::Context, _params: &mut Vec<Variable>) -> Option<VariableValue> {
        None
    }

    fn load_operations(builtins: &[(Sequence, Option<VariableType>, Builtin)]) -> (Automaton, Vec<OperationTemplate>) {
        let mut aut = Automaton::new();
        let mut ops = vec![];
        for (i,(seq,ret,op)) in builtins.into_iter().enumerate() {
            let operation = OperationTemplate::from_builtin(i, seq.clone(), Func::Builtin(*op), ret.clone());
            if !aut.register(seq.clone(), SequenceValue::Operation(operation.clone())) {
                panic!("error: union did not create any new states");
            }
            ops.push(operation);
        }
        (aut,ops)
    }

    impl Automaton {
        pub fn from(la: (Sequence, SequenceValue)) -> Self {
            let mut s = Self::new();
            s.register(la.0, la.1);
            s
        }
        pub fn len(&self) -> usize {
            self.states.len()
        }
    }

    fn verify(a: Automaton, expectations: HashMap<Sequence, Option<usize>>) {
        for (seq, val) in expectations {
            let actual_id = match a.run(seq.get()) {
                Some(SequenceValue::Operation(op)) => op.get_id(),
                Some(SequenceValue::Structure(s)) => s.get_id(),
                None => {
                    assert!(val.is_none(), "expected {}, but got none - sequence `{seq}`", val.unwrap());
                    continue;
                }
            };
            let Some(expected_id) = val else {
                panic!("got id `{actual_id}` but expected none - sequence `{seq}`");
            };
            assert_eq!(actual_id, expected_id, "expected id {expected_id}, but got {actual_id}");
        }
    }

    #[test]
    fn test_automaton_run() {
        let (a,_) = load_operations(&builtins!(
                ("a" [Int] "b"), nop;
        ));
        let expectations = expect!(
            ("a" [Int] "b") => Some(0);
            ("a" [Pos] "b") => None;
            ("a" [Int] "c") => None;
        );
        verify(a, expectations);
    }

    #[test]
    fn test_automaton_union() {
        let (mut a,_) = load_operations(&builtins!(
                ("a" [Int] "b"), nop;
                ));
        a.register(seq!("a" Pos "c"), SequenceValue::Operation(OperationTemplate::placeholder(1)));
        let expectations = expect!(
            ("a" [Int] "b") => Some(0);
            ("a" Pos "c") => Some(1);
            ("a" Pos "b") => None;
        );
        verify(a, expectations);
    }

    #[test]
    fn test_automaton_priority_choice() {
        let (a,_) = load_operations(&builtins!(
                ("a" Int "b"), nop;
                ("a" (Any(0)) "b"), nop;
        ));
        let expectations = expect!(
            ("a" Int "b") => Some(0); // chooses the first one
        );
        verify(a, expectations);
    }

    #[test]
    fn test_automaton_priority_choice_vec() {
        let (a,_) = load_operations(&builtins!(
                ("a" [Int] "b"), nop;
                ("a" (Any(0)) "b"), nop;
                ("a" [Any(0)] "b"), nop;
        ));
        let expectations = expect!(
            ("a" [Int] "b") => Some(0);
            ("a" Int "b") => Some(1);
            ("a" [Pos] "b") => Some(2);
        );
        verify(a, expectations);
    }

    #[test]
    fn test_automaton_backtrace_small() {
        let (a,_) = load_operations(&builtins!(
                ("a" Int "b"), nop;
                ("a" (Any(0)) "c"), nop;
        ));
        let expectations = expect!(
            ("a" Int "c") => Some(1);
        );
        verify(a, expectations);
    }

    #[test]
    fn test_automaton_backtrace_large() {
        let (a,_) = load_operations(&builtins!(
                ("a" Int "a" Int "a"), nop;
                ("a" (Any(0)) "a" Pos "a"), nop;
                ("a" (Any(0)) "a" Int "a"), nop;
                ("a" Int "a" (Any(0)) "b"), nop;
                ("a" Int "a" Pos "b"), nop; 
        ));
        let expectations = expect!(
            ("a" Int "a" Int "a") => Some(0);
            ("a" Int "a" Pos "a") => Some(1);
            ("a" Color "a" Int "a") => Some(2);
            ("a" Int "a" Int "b") => Some(3);
            ("a" Int "a" Pos "b") => Some(4);
        );
        verify(a, expectations);
    }

    #[test]
    fn test_get_interpretations() {
        // 1)
        let (a,_) = load_operations(&builtins!(
                ("a" (Any(0)) "b"), nop;
                ("a" Int "b"), nop;
                ("a" Pos "b"), nop;
        ));
        let paths = a.get_interpretations(seq!("a" (Any(0)) "b").get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(seq!("a" Int "b").get(), None, &vec![]);
        assert_eq!(paths.len(), 1);
        let paths = a.get_interpretations(seq!("a" Pos "b").get(), None, &vec![]);
        assert_eq!(paths.len(), 1);
        // 2)
        let (a,_) = load_operations(&builtins!(
                (a Int b Int c), nop;
                (a Int b (Any(1)) c), nop;
                (a Int b Pos c), nop;
                (a Pos b Int c), nop;
                (a Pos b Pos c), nop;
                (a (Any(1)) b (Any(1)) b), nop;
        ));
        let paths = a.get_interpretations(seq!(a (Any(0)) b (Any(0)) c).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        // 3)
        let (a,_) = load_operations(&builtins!(
                (a [Int] b), nop;
                (a [Pos] b), nop;
                (a [Any(0)] b), nop;
        ));
        let paths = a.get_interpretations(seq!(a (Any(0)) b).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(seq!(a [Any(0)] b).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(seq!(a [Int] b).get(), None, &vec![]);
        assert_eq!(paths.len(), 1);
        // 4)
        let (a,_) = load_operations(&builtins!(
                (a [Any(0)] (Any(0))), nop;
                (a [Any(0)] Int), nop;
                (a [Any(0)] Pos), nop;
        ));
        let paths = a.get_interpretations(seq!(a [Any(0)] (Any(0))).get(), None, &vec![]);
        assert_eq!(paths.len(), 3);
        let paths = a.get_interpretations(seq!(a (Any(0)) (Any(0))).get(), None, &vec![]);
        assert_eq!(paths.len(), 0);
        let paths = a.get_interpretations(seq!(a [Int] Color).get(), None, &vec![]);
        assert_eq!(paths, vec![]);
        // 5)
        let (a,ops) = load_operations(&builtins!(
                ("a" Int) => VariableType::Int, nop;
                ("a" [Int]) => VariableType::Int, nop;
                ("a" Pos) => VariableType::Pos, nop;
        ));
        let seq = seq!(a (Any(0)));
        assert_eq!(a.get_interpretations(seq.get(), None, &ops).len(), 3);
        assert_eq!(a.get_interpretations(seq.get(), Some(&VariableType::Any(1)), &ops).len(), 3);
        assert_eq!(a.get_interpretations(seq.get(), Some(&VariableType::Int), &ops).len(), 2);
    }
}
