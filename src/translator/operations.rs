use tree_sitter::Node;

use crate::{event::{Event, Operation}, translator::{SequenceValue, automata::{automaton::Automaton, linear_automaton::LinearAutomaton}, get_children, seq_to_str, translator::Kind, type_constraints::TypeConstraints}, variable::{Variable, stack::VariableMap, types::VariableType, values::VariableValue}};

use super::{translator::Translator, Sequence, Word};

// type Interperation = Vec<TypeConstraints>;
pub struct OperationRepr {
    pub signature: Sequence,
    pub params: Vec<String>,
    pub iterators: Vec<usize>,
    pub structure_param_id: Option<usize>,
}

impl OperationRepr {
    pub fn new( signature: Sequence, params: Vec<String>, iterators: Vec<usize>, structure_param_id: Option<usize>) -> Self {
        Self { signature, params, iterators, structure_param_id }
    }

    pub fn into_operation(self, id: usize, signature: Sequence, events: Vec<Event>, members: Vec<(String, SequenceValue,Vec<Variable>)>) -> Operation {
        Operation::new(id, signature, self.params, events, self.iterators, members, self.structure_param_id)
    }
}

impl Translator {
    pub fn parse_operation(&mut self, signature_node: &Node, definition_node: &Node) {
        let (signature, operands, iterators) = self.parse_lhs(signature_node);
        self.handle_operation(definition_node, &operands, &signature, iterators);
    }

    /// parses left hand side and returns:
    /// - its signature
    /// - variable names
    /// - iterators (only valid for operations)
    pub fn parse_lhs(&self, lhs: &Node) -> (Sequence, Vec<String>, Vec<usize>) {
        let mut seq = vec![];
        let mut operands = vec![];
        let mut has_main_iterator = false;
        let mut iterators: Vec<usize> = vec![];
        for elem in get_children(lhs) {
            match elem.kind() {
                "keyword" => {
                    seq.push(Word::Keyword(self.text(&elem).to_string()));
                }
                "variable" => {
                    let name = self.get_variable_name(&elem).unwrap();  // variable always has a valid name
                    push_variable_into_signature(name, &mut seq, &mut operands);
                }
                "iterator" => {
                    let var_node = elem.child_by_field_name("variable").expect("error: iterator without variable field");
                    var_node.expect_kind("variable", self);
                    let name = self.get_variable_name(&var_node).unwrap();  // variable always has a valid name
                    push_variable_into_signature(name, &mut seq, &mut operands);
                    let var_id = operands.len()-1;
                    if elem.child_by_field_name("main").is_some() {    // main iterator
                        assert!(has_main_iterator == false);    // FIXME: nice error message / warning
                        has_main_iterator = true;
                        iterators.insert(0, var_id);
                    } else {
                        iterators.push(var_id);
                    }
                }
                x => panic!("error: unexpected type {x} in sequence")
            }
        }
        (seq, operands, iterators)
    }

    /// gets all possible interpretations of an event node
    /// interpretation means: for a given sequence, what are the possible types for its ambiguous
    /// variables
    /// for efficiency, it returns whether at least one interpretation was found
    pub fn get_event_interpretations(&self, event_node: &Node, interpretations: &mut Vec<Vec<TypeConstraints>>) -> bool {
        let seq = self.get_sequence(event_node);
        // TODO: change so that it can also update static variables
        let ints = self.action_decision_automaton.get_interpretations(&seq, None);

        if ints.is_empty() {
            return false;
        }
        interpretations.push(ints);
        true
    }

    /// insert new operation in self.operations
    fn create_typed_operation(&mut self, signature: &Vec<Word>, operands: &Vec<String>, types: &Vec<VariableType>, events_node: &Node, iterators: &Vec<usize>, member_names: &Vec<String>) -> bool {
        // swap signature
        let mut new_signature = vec![];
        for w in signature {
            if let Word::Type(VariableType::Any(var_id)) = w {
                let v = &types[*var_id];
                if iterators.contains(var_id) {
                    new_signature.push(Word::Type(VariableType::Vec(Box::new(v.clone()))));
                } else {
                    new_signature.push(Word::Type(v.clone()));
                }
                let var_name;
                if operands.len() > *var_id {
                    var_name = &operands[*var_id];
                } else if operands.len()+member_names.len() > *var_id {
                    var_name = &member_names[*var_id-operands.len()];
                } else {
                    panic!("error: got variable id {var_id} with operands {operands:?} and members {member_names:?}");
                }
                self.globals.update_variable(var_name, v.default());
            } else {
                new_signature.push(w.clone());
            }
        }
        // swap event signatures and create events
        let mut op_events = vec![];
        let mut members = vec![];
        for event in get_children(events_node) {
            if event.kind() == "var_definition" {
                let name = self.get_var_definition_name(&event).to_string();
                let (seq,params) = self.get_sequence_with_params(&event.child_by_field_name("rhs").unwrap());
                let sv = self.action_decision_automaton.run(&seq).expect("error: unknown sequence");
                match sv {
                    SequenceValue::Operation(_) => {
                        todo!("operation returns");
                    }
                    SequenceValue::Component(id) => {
                        self.globals.update_variable(&name, VariableType::Component(id).default());
                    }
                    SequenceValue::Value(ref t) => {
                        self.globals.update_variable(&name, t.default());
                    }
                }
                members.push((name,sv.clone(),params));
            } else {
                op_events.push(self.get_event(&event));
            }
        }
        self.add_operation(new_signature, operands.clone(), op_events, iterators, members, None)
    }

    pub fn add_operation(&mut self, signature: Vec<Word>, operands: Vec<String>, events: Vec<Event>, iterators: &Vec<usize>, members: Vec<(String,SequenceValue,Vec<Variable>)>, str_id: Option<usize>) -> bool {
        // println!("adding operation '{}', {events:?}", seq_to_str(&signature));
        let op_id = self._number_of_builtin_operations + self.operations.len() + 1;
        let la = LinearAutomaton::new(signature.clone(), SequenceValue::Operation(op_id));
        if !self.action_decision_automaton.union(la) {
            return false
        }
        self.operations.insert(op_id, Operation::new(op_id, signature, operands, events, iterators.clone(), members, str_id));
        true
    }

    pub fn get_event(&mut self, event_node: &Node) -> Event {
        event_node.expect_kind("sequence", self);
        let mut params = vec![];
        let mut seq = vec![];
        for w in get_children(event_node) {
            match w.kind() {
                "keyword" => seq.push(Word::Keyword(self.text(&w).to_string())),
                "value" => {
                    let val = self.get_atomic_value(&w);
                    seq.push(Word::Type(val.get_type()));
                    if let Some(var_name) = self.get_variable_name(&w) {
                        params.push(Variable::new(var_name, val.get_type()));
                    } else {
                        params.push(Variable::new_static(val));
                    }
                }
                _ => panic!()
            }
        }
        let sv = self.action_decision_automaton.run(&seq).expect(&format!("error: failed to get sequence {:?}", seq));
        if let SequenceValue::Operation(x) = sv {
            let event;
            if self.is_builtin_operation(x) {
                event = Event::new(x, params, vec![], VariableMap::new())
            } else {
                event = self.operations.get(&x).expect(&format!("error: unknown operation {x}")).instantiate(params, &self.structures, &mut self.globals);
            }
            event
        } else {
            panic!("error: unexpected seq value {:?}", sv);
        }
    }

    pub fn infer_datatypes_from_interpretations(&mut self, interpretations: &Vec<Vec<TypeConstraints>>, signature: &Vec<Word>, operands: &Vec<String>, events_node: &Node, iterators: &Vec<usize>, variables: &Vec<String>) {
        if interpretations.len() == 0 { return; }
        self.infer_datatypes_from_interpretations_rec(&TypeConstraints::new(operands.len()), &interpretations, signature, operands, events_node, iterators, variables);
        self.resolve_variables(operands.len()+variables.len());
    }

    fn infer_datatypes_from_interpretations_rec(&mut self, interpretation: &TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Vec<Word>, operands: &Vec<String>, events_node: &Node, iterators: &Vec<usize>, variables: &Vec<String>) {
        if rest.is_empty() { 
            self.create_typed_operation(signature, operands, interpretation.get_types(), events_node, iterators, variables);
            return; 
        }
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                self.infer_datatypes_from_interpretations_rec(&prod, &rest[1..], signature, operands, events_node, iterators, variables);
            }
        }
    }

    pub fn parse_operation_definition(&mut self, definition_node: &Node, _signature: &Vec<Word>, aut: Option<&Automaton>) -> (Vec<String>,Vec<Vec<TypeConstraints>>) {
        let mut interpretations = vec![];
        let mut member_names = vec![];
        for e in get_children(definition_node) {
            let mut ints;
            match e.kind() {
                "var_definition" => {
                    let var_id = self.new_unresolved_variable();
                    let name = self.get_var_definition_name(&e).to_string();
                    member_names.push(name.to_string());
                    self.globals.add_variable(name.clone(), VariableValue::Any(var_id));
                    let seq = self.get_sequence(&e.child_by_field_name("rhs").unwrap());
                    if seq.len() == 1 && seq[0].is_type() {
                        let mut tc = TypeConstraints::_new();
                        tc.intersect_var(&seq[0].get_variable_type().unwrap(), var_id);
                        ints = vec![tc];
                    } else {
                        ints = self.action_decision_automaton.get_interpretations(&seq, Some(var_id));
                        if ints.len() == 0 && let Some(aut) = aut {
                            ints = aut.get_interpretations(&seq, Some(var_id));
                        }
                    }
                }
                "sequence" => {
                    let seq = self.get_sequence(&e);
                    ints = self.action_decision_automaton.get_interpretations(&seq, None);
                    if ints.len() == 0 && let Some(aut) = aut {
                        ints = aut.get_interpretations(&seq, None);
                    }
                }
                x => panic!("error: unexpected operation statement: {x}")   // TODO: friendlify
            }
            if ints.is_empty() { return (vec![],vec![]) }
            interpretations.push(ints);
        }
        (member_names,interpretations)
    }

    fn operation_member_assert(&self, variables: &Vec<String>, operands: &Vec<String>) -> bool {
        for n in variables {
            assert!(!operands.contains(&n));  // TODO: user friendlify
        }
        true
    }

    fn handle_operation(&mut self, definition_node: &Node, operands: &Vec<String>, signature: &Vec<Word>, iterators: Vec<usize>) {
        self.globals.push_layer();
        self.push_signature_params_to_scope(operands);
        let (variables,interpretations) = self.parse_operation_definition(definition_node, signature, None);
        if interpretations.is_empty() { println!("nothing for {}", seq_to_str(signature)); return }    // TODO: warning
        self.operation_member_assert(&variables, operands);
        self.infer_datatypes_from_interpretations(&interpretations, signature, operands, definition_node, &iterators, &variables);
        self.globals.pop_layer();
    }

    pub fn is_builtin_operation(&self, id: usize) -> bool {
        id <= self._number_of_builtin_operations
    }
}

fn push_variable_into_signature(var_name: &str, seq: &mut Sequence, operands: &mut Vec<String>) {
    seq.push(Word::Type(VariableType::Any(operands.len())));
    operands.push(var_name.to_string());
}

