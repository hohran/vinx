use crate::{action::{Action, Trigger}, event::Event, translator::{ast, error::CompilationError, parser::parser::Parser}};
use crate::context::Context;

impl Parser {
    pub fn parse_action(&mut self, action: &ast::Action) -> Result<(), CompilationError> {
        if !action.trigger.active && action.label.is_none() {
            return Ok(());
        }
        let trigger = Trigger::from(action.trigger.clone(), self.context.get_stack());
        let mut events = vec![];
        let mut locals = vec![];
        self.context.push_scope();
        for event in &action.events {
            match event {
                ast::Event::Operation(op) => events.push(Event::Call(self.parse_expression(op)?)),
                ast::Event::Assignment(assignment) => {
                    let value = self.parse_expression(&assignment.value.0)?;
                    let name = assignment.name.0.clone();
                    events.push(Event::Assignment(name, value));
                }
                // ast::Event::VarDefinition(var_def, _) => {
                //     let definition = self.get_var_definition(var_def, None)?;
                //     let (seq, params) = definition.get_value();
                //     let sv = self.get_sequence_value(seq)?;
                //     let seq_type = sv.get_return_type(&params.iter().map(|p| p.get_type()).collect()).unwrap();
                //     if !seq_type.is_assignable_to(definition.get_type()) {
                //         panic!("error: type {seq_type} is not assignable to {}", definition.get_type()) // TODO: friendlify
                //     }
                //     if !self.context.add_variable(definition.get_name().clone(), seq_type.default()) {
                //         let first_defined_range = action.find_variable_definition(definition.get_name());
                //         return Err(CompilationError::DuplicateMemberName(definition.get_name().to_string(), self.get_location(&var_def.name.1), self.get_location(&first_defined_range)))
                //     }
                //     locals.push((definition.get_name().clone(), seq_type.clone()));
                //     // TODO: what to do with call?
                //     events.push(Event::Assignment(definition.get_name().clone(), op));
                // }
                ast::Event::VarDefinition(var_def) => {
                    let definition = self.get_var_definition(var_def, None)?;
                    let expr = definition.get_value();
                    let Some(expr_var) = expr.evaluate_at_compiletime(&mut self.context) else {
                        panic!("error: expr has no return value");
                    };
                    let value = self.context.get_value(&expr_var);
                    if !self.context.add_variable(definition.get_name().clone(), value.clone()) {
                        let first_defined_range = action.find_variable_definition(definition.get_name());
                        return Err(CompilationError::DuplicateMemberName(definition.get_name().to_string(), self.get_location(&var_def.name.1), self.get_location(&first_defined_range)))
                    }
                    locals.push((definition.get_name().clone(), expr.get_type_unchecked()));
                    events.push(Event::Assignment(definition.get_name().clone(), expr.clone()));
                }
            }
        }
        self.context.pop_scope();
        let a = Action::new(action.label.clone().unwrap_or("".to_string()), events, trigger, locals);
        self.actions.push(a);
        Ok(())
    }

    // pub fn get_operation(&mut self, event: &ast::Sequence) -> Result<Operation, CompilationError> {
    //     let (seq, params) = self.parse_sequence(event)?;
    //     let sv = self.get_sequence_value(&seq)?;
    //     let SequenceValue::Operation(x) = sv else {
    //         // TODO: handle returning
    //         panic!("error: unexpected seq value {:?}", sv);
    //     };
    //     let event;
    //     event = self.operations[x].get().instantiate(params);
    //     Ok(event)
    // }
}
