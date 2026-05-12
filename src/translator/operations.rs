use tree_sitter::Node;

use crate::{context::Context, event::{Event, Operation}, translator::{SequenceValue, automata::Automaton, get_children, sequence::Sequence, signature::Signature, translator::Kind, type_constraints::TypeConstraints}, variable::{Variable, VariableType, VariableValue}};

use super::{translator::Translator, Word};

pub type MemberDef = (String, SequenceValue, Vec<Variable>);

impl Translator {
    pub fn parse_operation(&mut self, signature_node: &Node, definition_node: &Node) {
        let signature = self.parse_signature(signature_node);
        let (variables,interpretations) = self.parse_operation_definition(definition_node, &signature.sequence, None);
        if interpretations.is_empty() { println!("nothing for {signature}"); return }    // TODO: warning
        self.operation_member_assert(&variables, &signature.params);
        self.infer_datatypes_from_interpretations(&interpretations, &signature, definition_node, &variables);
    }

    /// insert new operation in self.operations
    fn create_typed_operation(&mut self, signature: &Signature, types: &Vec<VariableType>, events_node: &Node) -> bool {
        let mut new_signature = signature.clone();
        new_signature.swap_types(types);
        self.update_stack_with_signature(&new_signature);
        let (op_events,members) = self.get_operation_definition(events_node, None);
        self.add_operation(new_signature, op_events, members)
    }

    pub fn get_operation_definition(&mut self, definition_node: &Node, structure: Option<usize>) -> (Vec<Event>,Vec<MemberDef>) {
        let mut events = vec![];
        let mut members = vec![];
        for stmt in get_children(definition_node) {
            if stmt.kind() == "var_definition" {
                members.push(self.get_member(&stmt));
            } else {
                let mut event = self.get_event(&stmt);
                if let Some(structure_id) = structure {
                    let op = &self.operations[event.get_id()];
                    if let Some(struct_id) = op.method_of() {
                        if *struct_id == structure_id {
                            event.deactivate_struct();
                        }
                    }
                }
                events.push(event);
            }
        }
        (events,members)
    }

    pub fn get_member(&mut self, stmt: &Node) -> MemberDef {
        stmt.expect_kind("var_definition", self);
        let name = self.get_var_definition_name(&stmt).to_string();
        let (seq,params) = self.get_sequence_with_params(&stmt.child_by_field_name("rhs").unwrap());
        let sv = self.action_decision_automaton.run(seq.get()).expect("error: unknown sequence");
        self.globals.update_variable(&name, sv.into_variable_type(&self.operations).default());
        (name,sv.clone(),params)
    }

    pub fn add_operation(&mut self, signature: Signature, events: Vec<Event>, members: Vec<(String,SequenceValue,Vec<Variable>)>) -> bool {
        let op_id = self.operations.len();
        // println!("adding operation '{signature}' => {op_id} ({})", signature.sequence);
        if !self.action_decision_automaton.register(signature.sequence.clone(), SequenceValue::Operation(op_id)) {
            return false // TODO: generate warning
        }
        self.operations.push(Operation::new(op_id, signature.sequence, signature.params, events, signature.iterators, members, signature.structure_param_id, None));
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
            event = self.operations[x].instantiate(params, &mut Context::empty(), &self.operations, &self.structures, &mut self.globals);
            event
        } else {
            panic!("error: unexpected seq value {:?}", sv);
        }
    }

    pub fn infer_datatypes_from_interpretations(&mut self, interpretations: &Vec<Vec<TypeConstraints>>, signature: &Signature, events_node: &Node, variables: &Vec<String>) {
        if interpretations.len() == 0 { return; }
        self.infer_datatypes_from_interpretations_rec(&TypeConstraints::new(), &interpretations, signature, events_node, variables);
        self.resolve_variables(signature.params.len()+variables.len());
    }

    fn infer_datatypes_from_interpretations_rec(&mut self, interpretation: &TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Signature, events_node: &Node, variables: &Vec<String>) {
        if rest.is_empty() { 
            self.create_typed_operation(signature, interpretation.get_types(), events_node);
            return; 
        }
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                self.infer_datatypes_from_interpretations_rec(&prod, &rest[1..], signature, events_node, variables);
            }
        }
    }

    pub fn parse_operation_definition(&mut self, definition_node: &Node, _signature: &Sequence, aut: Option<&Automaton>) -> (Vec<String>,Vec<Vec<TypeConstraints>>) {
        let mut interpretations = vec![];
        let mut member_names = vec![];
        for e in get_children(definition_node) {
            let mut ints;
            match e.kind() {
                "var_definition" => {
                    let (var_id,seq) = self.add_member(&e, &mut member_names);
                    ints = self.action_decision_automaton.get_interpretations(seq.get(), Some(var_id), &self.operations);
                    if ints.len() == 0 && let Some(aut) = aut {
                        ints = aut.get_interpretations(seq.get(), Some(var_id), &self.operations);
                    }
                }
                "sequence" => {
                    let seq = self.get_sequence(&e);
                    ints = self.action_decision_automaton.get_interpretations(seq.get(), None, &self.operations);
                    if ints.len() == 0 && let Some(aut) = aut {
                        ints = aut.get_interpretations(seq.get(), None, &self.operations);
                    }
                }
                x => panic!("error: unexpected operation statement: {x}")   // TODO: friendlify
            }
            if ints.is_empty() { return (vec![],vec![]) }
            interpretations.push(ints);
        }
        (member_names,interpretations)
    }

    pub fn add_member(&mut self, node: &Node, member_names: &mut Vec<String>) -> (usize,Sequence) {
        node.expect_kind("var_definition", self);
        let var_id = self.new_unresolved_variable();
        let name = self.get_var_definition_name(&node).to_string();
        member_names.push(name.to_string());
        self.globals.add_variable(name.clone(), VariableValue::Any(var_id));
        let seq = self.get_sequence(&node.child_by_field_name("rhs").unwrap());
        (var_id,seq)
    }

    fn operation_member_assert(&self, variables: &Vec<String>, operands: &Vec<String>) -> bool {
        for n in variables {
            assert!(!operands.contains(&n));  // TODO: user friendlify
        }
        true
    }
}
