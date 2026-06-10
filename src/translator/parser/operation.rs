use crate::{event::OperationTemplate, translator::{MemberDef, SequenceValue, Signature, ast, automata::Automaton, error::CompilationError, parser::parser::Parser, sequence::StructureId, type_constraints::TypeConstraints}, variable::VariableValue};

impl Parser {
    pub fn parse_operation(&mut self, definition: &ast::Definition) -> Result<(), CompilationError> {
        let signature = self.parse_signature(&definition.signature)?;
        let (variables,interpretations) = self.parse_operation_definition(definition, None)?;
        if interpretations.is_empty() { println!("nothing for {signature}"); return Ok(()); }    // TODO: warning
        self.infer_datatypes_from_interpretations(&interpretations, &signature, definition, &variables)?;
        Ok(())
    }

    pub fn parse_operation_definition(&mut self, definition: &ast::Definition, aut: Option<&Automaton>) -> Result<(Vec<String>, Vec<Vec<TypeConstraints>>), CompilationError> {
        let mut interpretations = vec![];
        let mut member_names = vec![];
        for stmt in &definition.body {
            let mut ints;
            match stmt {
                ast::definition::Statement::VarDefinition(var_def) => {
                    let member_id = self.new_unresolved_variable();
                    member_names.push(var_def.name.clone());
                    if !self.globals.add_variable(var_def.name.clone(), VariableValue::Any(member_id)) {
                        return Err(CompilationError::TemporaryError(format!("duplicate member name in operation definition: {}", var_def.name)));
                    }
                    let (seq,_) = self.get_sequence(&var_def.value)?;
                    ints = self.automaton.get_interpretations(seq.get(), Some(member_id), &self.operations);
                    if ints.len() == 0 && let Some(aut) = aut {
                        ints = aut.get_interpretations(seq.get(), Some(member_id), &self.operations);
                    }
                }
                ast::definition::Statement::Event(e) => {
                    let (seq,_) = self.get_sequence(&e)?;
                    ints = self.automaton.get_interpretations(seq.get(), None, &self.operations);
                    if ints.len() == 0 && let Some(aut) = aut {
                        ints = aut.get_interpretations(seq.get(), None, &self.operations);
                    }
                }
                ast::definition::Statement::Definition(_) => panic!("error: nested definition not expected in operation")
            }
            if ints.is_empty() { return Ok((vec![],vec![])) }
            interpretations.push(ints);
        }
        Ok((member_names,interpretations))
    }

    /// Infer the possible interpretations of operation given by `signature`, events in
    /// `events_node` and local `members`.
    /// These interpretations are then stored in the Translator with `add_operation`.
    pub fn infer_datatypes_from_interpretations(&mut self, interpretations: &Vec<Vec<TypeConstraints>>, signature: &Signature, definition: &ast::Definition, members: &Vec<String>) -> Result<(), CompilationError>{
        if interpretations.len() == 0 { return Ok(()); }
        self.infer_datatypes_from_interpretations_rec(TypeConstraints::new(), &interpretations, signature, definition, members)?;
        self.resolve_variables(signature.params.len()+members.len());
        Ok(())
    }

    /// Recursively infer the operation interpretations.
    ///
    /// `interpretation` holds the current interpretation and `rest` holds the possible
    /// interpretations for the rest of unprocessed events.
    /// During one call of this function, one event (its interpretations) is processed, potentially
    /// generating multiple more interpretations.
    fn infer_datatypes_from_interpretations_rec(&mut self, interpretation: TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Signature, definition: &ast::Definition, variables: &Vec<String>) -> Result<(), CompilationError> {
        if rest.is_empty() { 
            self.create_typed_operation(signature, interpretation, definition)?;
            return Ok(()); 
        }
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                self.infer_datatypes_from_interpretations_rec(prod, &rest[1..], signature, definition, variables)?;
            }
        }
        Ok(())
    }

    /// Create an operation with `signature` based on a `interpretation`.
    /// Register it in the Translator automaton.
    fn create_typed_operation(&mut self, signature: &Signature, interpretation: TypeConstraints, definition: &ast::Definition) -> Result<bool, CompilationError> {
        let mut new_signature = signature.clone();
        new_signature.swap_types(interpretation.get_types());
        self.update_stack_with_signature(&new_signature);
        let (op_events,members) = self.get_operation_definition(definition, None)?;
        Ok(self.add_operation(new_signature, op_events, members))
    }

    /// Add given operation to the global list and register its signature in the Translator's automaton.
    pub fn add_operation(&mut self, signature: Signature, events: Vec<Operation>, members: Vec<MemberDef>) -> bool {
        let op_id = self.operations.len();
        // println!("adding operation '{signature}' => {op_id} ({})", signature.sequence);
        if !self.automaton.register(signature.sequence.clone(), SequenceValue::Operation(op_id)) {
            return false // TODO: generate warning
        }
        self.operations.push(OperationTemplate::new(op_id, signature, events, members, None));
        true
    }

    /// Get a list of events and member definitions for an operation definition.
    /// If the operation is a method, `structure` is set with respective id.
    /// 
    /// All parameters of the processed operation must be set to the correct type on the global stack.
    pub fn get_operation_definition(&mut self, definition: &ast::Definition, structure: Option<StructureId>) -> Result<(Vec<Operation>,Vec<MemberDef>), CompilationError> {
        let mut events = vec![];
        let mut members = vec![];
        for stmt in &definition.body {
            match stmt {
                ast::definition::Statement::VarDefinition(d) => members.push(self.get_member_definition(d)?),
                ast::definition::Statement::Event(e) => {
                    let mut event = self.get_event(e)?;
                    // if this operation is a method
                    // and it contains another method
                    // of this structure: we do not
                    // have to load the structure
                    // parameters again
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
                ast::definition::Statement::Definition(_) => panic!("error: nested definition not expected in operation")
            }
        }
        Ok((events,members))
    }

    /// Get a member definition from an operation statement (e.g., $member = something).
    pub fn get_member_definition(&mut self, var_def: &ast::VarDefinition) -> Result<MemberDef, CompilationError> {
        let (seq,params) = self.get_sequence(&var_def.value)?;
        let Some(sv) = self.automaton.run(seq.get()) else {
            return Err(CompilationError::UnknownSequence(seq, self.placeholder_location()));
        };
        self.globals.update_variable(&var_def.name, sv.into_type(&self.operations).default());
        Ok((var_def.name.clone(),sv.clone(),params))
    }

    /// Update the type of every `signature` parameter on the stack.
    pub fn update_stack_with_signature(&mut self, signature: &Signature) {
        signature.foreach(|p,t| self.globals.update_variable(p, t.default()));
    }
}
