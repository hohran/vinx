use tree_sitter::Node;

use super::*;
use crate::{context::Context, event::{Event, OperationTemplate, Operations}, variable::{Variable, VariableValue}};

pub type MemberDef = (String, SequenceValue, Vec<Variable>);

impl Translator {
    /// Parse operation defined by given nodes, and store all its interpretations in the Translator.
    pub fn parse_operation(&mut self, signature_node: &Node, definition_node: &Node) -> Result<(), CompilationError> {
        let signature = self.get_signature(signature_node)?;
        let (variables,interpretations) = self.parse_operation_definition(definition_node, None, &|_,_| Ok(()))?;
        if interpretations.is_empty() { println!("nothing for {signature}"); return Ok(()); }    // TODO: warning
        self.operation_member_assert(&variables, &signature.params);
        self.infer_datatypes_from_interpretations(&interpretations, &signature, definition_node, &variables)?;
        Ok(())
    }

    /// Get a list of member names and a list of possible events interpretations for an operation
    /// definition given by `definition_node`.
    /// `aut` is an optional fallback automaton for events.
    pub fn parse_operation_definition<F>(&mut self, definition_node: &Node, aut: Option<&Automaton>, member_name_constraint: &F)-> Result<(Vec<String>,Vec<Vec<TypeConstraints>>), CompilationError>
        where F: Fn(&str, &Node) -> Result<(), CompilationError> {
            let mut interpretations = vec![];
            let mut member_names = vec![];
            for e in get_children(definition_node) {
                let mut ints;
                match e.kind() {
                    "var_definition" => {
                        let (var_id,seq) = self.parse_member(&e, &mut member_names, member_name_constraint)?;
                        ints = self.automaton.get_interpretations(seq.get(), Some(var_id), &self.operations);
                        if ints.len() == 0 && let Some(aut) = aut {
                            ints = aut.get_interpretations(seq.get(), Some(var_id), &self.operations);
                        }
                        println!("{seq} => {ints:?}");
                    }
                    "sequence" => {
                        let seq = self.get_sequence(&e);
                        ints = self.automaton.get_interpretations(seq.get(), None, &self.operations);
                        if ints.len() == 0 && let Some(aut) = aut {
                            ints = aut.get_interpretations(seq.get(), None, &self.operations);
                        }
                        println!("{seq} => {ints:?}");
                    }
                    x => panic!("error: unexpected operation statement: {x}")
                }
                if ints.is_empty() { return Ok((vec![],vec![])) }
                interpretations.push(ints);
            }
            Ok((member_names,interpretations))
        }

    /// Parse a member from `node`:
    ///  * add its name to the list `member_names`
    ///  * return its id and the sequence that initializes it
    pub fn parse_member<F>(&mut self, node: &Node, member_names: &mut Vec<String>, member_name_constraint: &F) -> Result<(usize,Sequence), CompilationError>
        where F: Fn(&str, &Node) -> Result<(), CompilationError> {
            self.expect_node_kind(node, "var_definition");
            let var_id = self.new_unresolved_variable();
            let name = self.get_var_definition_name(&node).to_string();
            member_name_constraint(&name, node)?;
            member_names.push(name.to_string());
            self.globals.add_variable(name.clone(), VariableValue::Any(var_id));
            let seq = self.get_sequence(&node.child_by_field_name("rhs").unwrap());
            Ok((var_id,seq))
    }

    /// Create an operation with `signature` based on a `interpretation`.
    /// Register it in the Translator automaton.
    fn create_typed_operation(&mut self, signature: &Signature, interpretation: TypeConstraints, events_node: &Node) -> Result<bool, CompilationError> {
        let mut new_signature = signature.clone();
        new_signature.swap_types(interpretation.get_types());
        self.update_stack_with_signature(&new_signature);
        let (op_events,members) = self.get_operation_definition(events_node, None)?;
        Ok(self.add_operation(new_signature, op_events, members))
    }

    /// Get a list of events and member definitions for an operation definition.
    /// If the operation is a method, `structure` is set with respective id.
    /// 
    /// All parameters of the processed operation must be set to the correct type on the global stack.
    pub fn get_operation_definition(&mut self, definition_node: &Node, structure: Option<StructureId>) -> Result<(Vec<Event>,Vec<MemberDef>), CompilationError> {
        let mut events = vec![];
        let mut members = vec![];
        for stmt in get_children(definition_node) {
            if stmt.kind() == "var_definition" {
                members.push(self.get_member_definition(&stmt)?);
            } else {
                let mut event = self.get_event(&stmt)?;
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
        Ok((events,members))
    }

    /// Get a member definition from an operation statement (e.g., $member = something).
    pub fn get_member_definition(&mut self, stmt: &Node) -> Result<MemberDef, CompilationError> {
        self.expect_node_kind(stmt, "var_definition");
        let name = self.get_var_definition_name(&stmt).to_string();
        let (seq,params) = self.get_sequence_with_params(&stmt.child_by_field_name("rhs").unwrap());
        let Some(sv) = self.automaton.run(seq.get()) else {
            return Err(CompilationError::UnknownSequence(seq, self.get_location(stmt)));
        };
        self.globals.update_variable(&name, sv.into_type(&self.operations).default());
        Ok((name,sv.clone(),params))
    }

    /// Add given operation to the global list and register its signature in the Translator's automaton.
    pub fn add_operation(&mut self, signature: Signature, events: Vec<Event>, members: Vec<MemberDef>) -> bool {
        let op_id = self.operations.len();
        // println!("adding operation '{signature}' => {op_id} ({})", signature.sequence);
        if !self.automaton.register(signature.sequence.clone(), SequenceValue::Operation(op_id)) {
            return false // TODO: generate warning
        }
        self.operations.push(OperationTemplate::new(op_id, signature, events, members, None));
        true
    }

    /// Get event from a node.
    pub fn get_event(&mut self, event_node: &Node) -> Result<Event, CompilationError> {
        let (seq, params) = self.get_sequence_with_params(event_node);
        let Some(sv) = self.automaton.run(seq.get()) else {
            return Err(CompilationError::UnknownSequence(seq, self.get_location(event_node)));
        };
        let SequenceValue::Operation(x) = sv else {
            // TODO: handle returning
            panic!("error: unexpected seq value {:?}", sv);
        };
        let event;
        event = self.operations[x].instantiate(params, &mut Context::empty(), &self.operations, &self.structures, &mut self.globals);
        Ok(event)
    }

    /// Infer the possible interpretations of operation given by `signature`, events in
    /// `events_node` and local `members`.
    /// These interpretations are then stored in the Translator with `add_operation`.
    pub fn infer_datatypes_from_interpretations(&mut self, interpretations: &Vec<Vec<TypeConstraints>>, signature: &Signature, events_node: &Node, members: &Vec<String>) -> Result<(), CompilationError>{
        if interpretations.len() == 0 { return Ok(()); }
        self.infer_datatypes_from_interpretations_rec(TypeConstraints::new(), &interpretations, signature, events_node, members)?;
        self.resolve_variables(signature.params.len()+members.len());
        Ok(())
    }

    /// Recursively infer the operation interpretations.
    ///
    /// `interpretation` holds the current interpretation and `rest` holds the possible
    /// interpretations for the rest of unprocessed events.
    /// During one call of this function, one event (its interpretations) is processed, potentially
    /// generating multiple more interpretations.
    fn infer_datatypes_from_interpretations_rec(&mut self, interpretation: TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Signature, events_node: &Node, variables: &Vec<String>) -> Result<(), CompilationError> {
        if rest.is_empty() { 
            self.create_typed_operation(signature, interpretation, events_node)?;
            return Ok(()); 
        }
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                self.infer_datatypes_from_interpretations_rec(prod, &rest[1..], signature, events_node, variables)?;
            }
        }
        Ok(())
    }

    /// Check that operation members are valid:
    ///  * They do not colide with any parameter name.
    fn operation_member_assert(&self, variables: &Vec<String>, operands: &Vec<String>) -> bool {
        for n in variables {
            assert!(!operands.contains(&n));  // TODO: user friendlify
        }
        true
    }
}
