use tree_sitter::Node;

use crate::{event::{Event, Operation}, translator::{SequenceValue, automata::linear_automaton::LinearAutomaton, get_children, translator::Kind, type_constraints::TypeConstraints}, variable::{Variable, stack::VariableMap, types::VariableType, values::VariableValue}};

use super::{translator::Translator, Sequence, Word};

// type Interperation = Vec<TypeConstraints>;

impl Translator {
    pub fn parse_operation(&mut self, node: &Node) {
        node.expect_kind("declaration", self);
        let children = get_children(node);
        let (signature, operands, iterators) = self.parse_lhs(&children[0]);
        let operation_node = children[1];
        self.handle_operation(&operation_node, &operands, &signature, iterators);
    }

    /// parses left hand side and returns:
    /// - its signature
    /// - variable names
    /// - iterators (only valid for operations)
    fn parse_lhs(&self, lhs: &Node) -> (Sequence, Vec<String>, Vec<usize>) {
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
                        assert!(has_main_iterator == false);
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
    fn get_event_interpretations(&self, node: &Node, var_count: usize, interpretations: &mut Vec<Vec<TypeConstraints>>) -> bool {
        let seq = self.get_sequence(node);
        let ints = self.action_decision_automaton.get_interpretations(&seq, var_count);
        if ints.is_empty() {
            return false;
        }
        interpretations.push(ints);
        true
    }

    /// insert new operation in self.operations
    fn create_typed_operation(&mut self, signature: &Vec<Word>, operands: &Vec<String>, types: &Vec<VariableType>, events_node: &Node, iterators: &Vec<usize>, variables: &VariableMap) -> bool {
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
                self.globals.update_variable(&operands[*var_id], v.default());
            } else {
                new_signature.push(w.clone());
            }
        }
        // swap event signatures and create events
        let mut op_events = vec![];
        for event in get_children(events_node) {
            if event.kind() == "var_definition" {
                continue;
            } else {
                op_events.push(self.get_event(&event));
            }
        }
        self.add_operation(new_signature, operands.clone(), op_events, iterators, variables)
    }

    fn get_event(&self, event_node: &Node) -> Event {
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
            if self.is_builtin_operation(*x) {
                event = Event::new(*x, params, vec![], VariableMap::new())
            } else {
                event = self.operations.get(x).expect(&format!("error: unknown operation {x}")).instantiate(params);
            }
            event
        } else {
            panic!("error: unexpected seq value {:?}", sv);
        }
    }

    fn infer_datatypes_from_interpretations(&mut self, interpretations: &Vec<Vec<TypeConstraints>>, signature: &Vec<Word>, operands: &Vec<String>, events_node: &Node, iterators: &Vec<usize>, variables: &VariableMap) {
        if interpretations.len() == 0 { return; }
        self.infer_datatypes_from_interpretations_rec(&TypeConstraints::new(operands.len()), &interpretations, signature, operands, events_node, iterators, variables);
    }

    fn infer_datatypes_from_interpretations_rec(&mut self, interpretation: &TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Vec<Word>, operands: &Vec<String>, events_node: &Node, iterators: &Vec<usize>, variables: &VariableMap) {
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

    /// TODO: first get all interpretations, then see if any is valid. if not maybe throw error
    fn handle_operation(&mut self, events_node: &Node, operands: &Vec<String>, signature: &Vec<Word>, iterators: Vec<usize>) {
        // push variables to scope
        self.globals.push_layer();
        for i in 0..operands.len() {
            let v = &operands[i];
            self.globals.add_variable(v.to_string(), VariableValue::Any(i));
        }
        // get possible event interpretations
        let mut interpretations = vec![]; // store of sequences, and for each
        let mut variables = VariableMap::new();
        for e in get_children(events_node) {
            if e.kind() == "var_definition" {
                let (name,val) = self.get_var_definition(&e);
                variables.insert(name,val);
                continue;
            }
            if !self.get_event_interpretations(&e, operands.len(), &mut interpretations) {
                panic!("error: no viable interpretations for operation {:?}", signature);
            }
        }
        if interpretations.len() == 0 {
            // empty declaration
            self.create_typed_operation(signature, operands, TypeConstraints::new(operands.len()).get_types(), events_node, &iterators, &variables);
            return;
        }
        self.infer_datatypes_from_interpretations(&interpretations, signature, operands, events_node, &iterators, &variables);
        // revert scope
        self.globals.pop_layer();
    }

    fn add_operation(&mut self, signature: Vec<Word>, operands: Vec<String>, events: Vec<Event>, iterators: &Vec<usize>, variables: &VariableMap) -> bool {
        let op_id = self._number_of_builtin_operations + self.operations.len() + 1;
        // println!("adding operation with signature {:?} as {op_id}", signature);
        let la = LinearAutomaton::new(signature, SequenceValue::Operation(op_id));
        if !self.action_decision_automaton.union(la) {
            return false
        }
        self.operations.insert(op_id, Operation::new(op_id, operands, events, iterators.clone(), variables.clone()));
        true
    }

    pub fn is_builtin_operation(&self, id: usize) -> bool {
        id <= self._number_of_builtin_operations
    }
}

fn push_variable_into_signature(var_name: &str, seq: &mut Sequence, operands: &mut Vec<String>) {
    seq.push(Word::Type(VariableType::Any(operands.len())));
    operands.push(var_name.to_string());
}

