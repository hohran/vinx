use crate::{action::{Action, Trigger}, context::Context, event::{Event, Operation}, translator::{SequenceValue, ast, error::CompilationError, parser::parser::Parser}};

impl Parser {
    pub fn parse_action(&mut self, action: &ast::Action) -> Result<(), CompilationError> {
        if !action.trigger.active && action.label.is_none() {
            return Ok(());
        }
        let trigger = Trigger::from(action.trigger.clone(), &self.globals);
        let mut events = vec![];
        for event in &action.events {
            match event {
                ast::Event::Operation(op, _) => events.push(Event::Call(self.get_event(op)?)),
                ast::Event::Assignment(assignment, _) => {
                    let operation = self.get_event(&assignment.value)?;
                    let name = assignment.name.clone();
                    events.push(Event::Assignment(name, operation));
                }
            }
            
        }
        let a = Action::new(action.label.clone().unwrap_or("".to_string()), events, trigger);
        self.actions.push(a);
        Ok(())
    }

    pub fn get_event(&mut self, event: &ast::Sequence) -> Result<Operation, CompilationError> {
        let (seq, params) = self.get_sequence(event)?;
        let Some(sv) = self.automaton.run(seq.get()) else {
            return Err(CompilationError::UnknownSequence(seq, self.placeholder_location()));
        };
        let SequenceValue::Operation(x) = sv else {
            // TODO: handle returning
            panic!("error: unexpected seq value {:?}", sv);
        };
        let event;
        event = self.operations[x].instantiate(params, &mut Context::empty(), &self.operations, &self.structures, &mut self.globals);
        Ok(event)
    }
}
