use crate::{action::{Action, Trigger}, event::{Event, Operation}, translator::{SequenceValue, ast::{self, Range}, error::CompilationError, parser::parser::Parser}};

impl Parser {
    pub fn parse_action(&mut self, action: &ast::Action) -> Result<(), CompilationError> {
        if !action.trigger.active && action.label.is_none() {
            return Ok(());
        }
        let trigger = Trigger::from(action.trigger.clone(), &self.globals);
        let mut events = vec![];
        let mut locals = vec![];
        self.globals.push();
        for event in &action.events {
            match event {
                ast::Event::Operation(op, _) => events.push(Event::Call(self.get_operation(op)?)),
                ast::Event::Assignment(assignment, _) => {
                    let operation = self.get_operation(&assignment.value.0)?;
                    let name = assignment.name.0.clone();
                    events.push(Event::Assignment(name, operation));
                }
                ast::Event::VarDefinition(var_def, _) => {
                    let definition = self.get_var_definition(var_def, None)?;
                    let (seq, params) = definition.get_value();
                    let Some(sv) = self.automaton.run(seq.get()) else {
                        panic!() // TODO: friendlify
                    };
                    let SequenceValue::Operation(op_id) = sv else {
                        panic!() // TODO: FIXME
                    };
                    let op = self.operations[op_id].get().instantiate(params.clone());
                    let Some(return_type) = op.get_return_type() else {
                        panic!() // TODO: friendlify
                    };
                    if !return_type.is_assignable_to(definition.get_type()) {
                        panic!("error: type {return_type} is not assignable to {}", definition.get_type()) // TODO: friendlify
                    }
                    if !self.globals.add_variable(definition.get_name().clone(), return_type.default()) {
                        let first_defined_range = action.find_variable_definition(definition.get_name());
                        return Err(CompilationError::DuplicateMemberName(definition.get_name().to_string(), self.get_location(&var_def.name.1), self.get_location(&first_defined_range)))
                    }
                    locals.push((definition.get_name().clone(), return_type.clone()));
                    events.push(Event::Assignment(definition.get_name().clone(), op));
                }
            }
        }
        self.globals.pop();
        let a = Action::new(action.label.clone().unwrap_or("".to_string()), events, trigger, locals);
        self.actions.push(a);
        Ok(())
    }

    pub fn get_operation(&mut self, event: &ast::Sequence) -> Result<Operation, CompilationError> {
        let (seq, params) = self.parse_sequence(event)?;
        let Some(sv) = self.automaton.run(seq.get()) else {
            return Err(CompilationError::UnknownSequence(seq, self.get_location(&Range::from(event))));
        };
        let SequenceValue::Operation(x) = sv else {
            // TODO: handle returning
            panic!("error: unexpected seq value {:?}", sv);
        };
        let event;
        event = self.operations[x].get().instantiate(params);
        Ok(event)
    }
}
