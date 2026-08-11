// TODO TODO TODO: assignments and variable definitions in actions/operations disallow to have a
// sequence which returns a structure now! FIXME FIXME FIXME
use crate::{event::{Event, Operation, OperationTemplate, OperationTemplateEnum}, translator::{SequenceValue, Signature, ast::{self, Range}, automata::Automaton, error::{CompilationError, Warning}, parser::parser::Parser, sequence::{SequenceType, StructureId}, type_constraints::TypeConstraints}, variable::VariableType};

pub type OperationMember = (String, VariableType); // name, type

impl Parser {
    pub fn parse_operation(&mut self, definition: &ast::Definition) -> Result<(), CompilationError> {
        let signature = self.parse_signature(&definition.signature)?;
        let (members,interpretations) = self.parse_operation_definition(definition, None)?;
        if interpretations.is_empty() {
            self.warnings.push(Warning::OperationWithoutInterpretation(signature.clone(), self.get_location(&Range::from(definition))));
            return Ok(())
        }
        self.check_members(&members, definition)?;
        self.infer_datatypes_from_interpretations(&interpretations, &signature, definition, &members)?;
        Ok(())
    }

    /// Throw compilation error when a variable is used before it is defined (as a member)
    /// The current implementation forbids for a global variable to be firstly used and later
    /// shadowed during the same operation.
    ///
    /// For example:
    /// ```vinx
    /// $x := 1;
    /// do something := {
    ///     process $x; // this should use the global $x
    ///     $x := "hello"; // this shadows $x with a string variable
    /// }
    /// ```
    ///
    fn check_members(&self, members: &Vec<String>, definition: &ast::Definition) -> Result<(), CompilationError> {
        let mut defined = vec![false; members.len()];
        let check = |defined: &Vec<bool>, var: &String, range: &Range| {
            if let Some(member_id) = members.iter().position(|x| x == var) {
                if !defined[member_id] {
                    return Err(CompilationError::MemberUsedBeforeDefinition(var.clone(), self.get_location(range)));
                }
            }
            Ok(())
        };
        for stmt in &definition.body {
            match &stmt.0 {
                ast::definition::Statement::VarDefinition(var_def) => {
                    let Some(member_id) = members.iter().position(|x| x == &var_def.name.0) else {
                        panic!();
                    };
                    defined[member_id] = true;
                }
                ast::definition::Statement::Assignment(ass) => {
                    check(&defined, &ass.name.0, &ass.name.1)?;
                    for (word, range) in &ass.value.0 {
                        let ast::sequence::Word::Value(ast::Value::Variable(name)) = word else {
                            continue;
                        };
                        check(&defined, name, range)?;
                    }
                }
                ast::definition::Statement::Event(event) => {
                    for (word, range) in event {
                        let ast::sequence::Word::Value(ast::Value::Variable(name)) = word else {
                            continue;
                        };
                        check(&defined, name, range)?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn get_operation_members(&self, definition: &ast::Definition) -> Vec<String> {
        let mut out = vec![];
        for stmt in &definition.body {
            let ast::definition::Statement::VarDefinition(var_def) = &stmt.0 else {
                continue;
            };
            out.push(var_def.name.0.clone());
        }
        out
    }

    pub fn parse_operation_definition(&mut self, definition: &ast::Definition, aut: Option<&Automaton>) -> Result<(Vec<String>, Vec<Vec<TypeConstraints>>), CompilationError> {
        let mut interpretations = vec![];
        let mut member_names = vec![];
        for stmt in &definition.body {
            let mut ints;
            match &stmt.0 {
                ast::definition::Statement::VarDefinition(var_def) => {
                    let member_id = self.new_unresolved_variable();
                    let member = self.get_var_definition(var_def, Some(member_id))?;
                    let member_type = VariableType::Any(member_id);
                    let member_name = var_def.name.0.to_string();
                    member_names.push(member_name.clone());
                    // let (value, value_range) = &var_def.value;
                    // let (seq,_) = self.parse_sequence(&value)?;
                    let (seq, _) = member.get_value();
                    ints = self.automaton.get_interpretations(seq.get(), Some(&member_type), &self.operations);
                    if ints.len() == 0 && let Some(aut) = aut {
                        ints = aut.get_interpretations(seq.get(), Some(&member_type), &self.operations);
                    }
                    if !self.globals.add_variable(member_name.clone(), member_type.default()) {
                        let first_defined_range = definition.find_variable_definition(&member_name);
                        return Err(CompilationError::DuplicateMemberName(member_name, self.get_location(&var_def.name.1), self.get_location(&first_defined_range)))
                    }
                }
                ast::definition::Statement::Assignment(var_def) => {
                    let member_name = var_def.name.0.to_string();
                    let Some(member_value) = self.globals.get_variable(&member_name) else {
                        panic!("error: no variable {}", member_name); // TODO: friendlify
                    };
                    let (value,_) = &var_def.value;
                    let (seq,_) = self.parse_sequence(&value)?;
                    ints = self.automaton.get_interpretations(seq.get(), Some(&member_value.get_type()), &self.operations);
                    if ints.len() == 0 && let Some(aut) = aut {
                        ints = aut.get_interpretations(seq.get(), Some(&member_value.get_type()), &self.operations);
                    }
                }
                ast::definition::Statement::Event(e) => {
                    let (seq,_) = self.parse_sequence(&e)?;
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
    fn infer_datatypes_from_interpretations_rec(&mut self, interpretation: TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Signature, definition: &ast::Definition, members: &Vec<String>) -> Result<(), CompilationError> {
        if rest.is_empty() { 
            self.create_typed_operation(signature, interpretation, definition, members)?;
            return Ok(()); 
        }
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                self.infer_datatypes_from_interpretations_rec(prod, &rest[1..], signature, definition, members)?;
            }
        }
        Ok(())
    }

    /// Create an operation with `signature` based on a `interpretation`.
    /// Register it in the Translator automaton.
    fn create_typed_operation(&mut self, signature: &Signature, interpretation: TypeConstraints, definition: &ast::Definition, members: &Vec<String>) -> Result<bool, CompilationError> {
        let mut new_signature = signature.clone();
        new_signature.swap_types(interpretation.get_types());
        self.update_stack_with_signature(&new_signature);
        self.update_stack_with_members(new_signature.sequence.get_types().len(), &interpretation, members);
        let op_events = self.get_operation_definition(definition, None)?;
        Ok(self.add_operation(new_signature, op_events, self.get_members(members)))
    }

    fn update_stack_with_members(&mut self, already_set: usize, interpretation: &TypeConstraints, members: &Vec<String>) {
        for (i,member) in members.iter().enumerate() {
            let index = already_set+i;
            let member_type = interpretation.at(index);
            self.globals.update_variable(member, member_type.default());
        }
    }

    pub fn get_members(&self, members: &Vec<String>) -> Vec<OperationMember> {
        members.iter().map(|name| 
            (name.clone(), self.globals.get_variable(name).expect(&format!("error: variable `{name}` not found")).get_type()))
            .collect()
    }

    /// Add given operation to the global list and register its signature in the Translator's automaton.
    pub fn add_operation(&mut self, signature: Signature, events: Vec<Event>, members: Vec<OperationMember>) -> bool {
        let op_id = self.operations.len();
        // println!("adding operation '{signature}' => {op_id} ({})", signature.sequence);
        if !self.automaton.register(signature.sequence.clone(), SequenceType::Operation) {
            return false // TODO: generate warning
        }
        let mut result = None;
        if !events.is_empty() {
            let last_event = &events[events.len()-1];
            if let Event::Call(event) = last_event {
                result = event.get_return_type().cloned();
            }
        }
        self.operations.push(OperationTemplateEnum::Standard(OperationTemplate::new(op_id, signature, events, members, result)));
        true
    }

    /// Get a list of events and member definitions for an operation definition.
    /// If the operation is a method, `structure` is set with respective id.
    /// 
    /// All parameters of the processed operation must be set to the correct type on the global stack.
    pub fn get_operation_definition(&mut self, definition: &ast::Definition, structure: Option<StructureId>) -> Result<Vec<Event>, CompilationError> {
        let mut events = vec![];
        for stmt in &definition.body {
            match &stmt.0 {
                ast::definition::Statement::Assignment(d) => {
                    let event = self.get_operation_event(&d.value.0, structure)?;
                    events.push(Event::Assignment(d.name.0.clone(), event));
                }
                ast::definition::Statement::VarDefinition(d) => {
                    let definition = self.get_var_definition(d, None)?;
                    let (seq,params) = definition.get_value();
                    let Some(SequenceValue::Operation(op_id)) = self.automaton.run(seq.get()) else {
                        panic!() // TODO: friendlify
                    };
                    let mut event = self.operations[op_id].get().instantiate(params.clone());
                    self.deactivate_struct_for_event(&mut event, structure);
                    events.push(Event::Assignment(d.name.0.clone(), event));
                }
                ast::definition::Statement::Event(e) => {
                    let event = self.get_operation_event(e, structure)?;
                    events.push(Event::Call(event));
                }
                ast::definition::Statement::Definition(_) => panic!("error: nested definition not expected in operation")
            }
        }
        Ok(events)
    }

    fn get_operation_event(&mut self, seq: &ast::Sequence, structure: Option<StructureId>) -> Result<Operation, CompilationError> {
        let mut event = self.get_operation(seq)?;
        self.deactivate_struct_for_event(&mut event, structure);
        Ok(event)
    }

    fn deactivate_struct_for_event(&self, event: &mut Operation, structure: Option<StructureId>) {
        // if this operation is a method
        // and it contains another method
        // of this structure: we do not
        // have to load the structure
        // parameters again
        if let Some(structure_id) = structure {
            let op = &self.operations[event.get_id()].get();
            if let Some(struct_id) = op.method_of() {
                if *struct_id == structure_id {
                    event.deactivate_struct();
                }
            }
        }
    }

    /// Update the type of every `signature` parameter on the stack.
    pub fn update_stack_with_signature(&mut self, signature: &Signature) {
        signature.foreach(|p,t| self.globals.update_variable(p, t.default()));
    }
}
