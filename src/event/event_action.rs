use crate::{context, event::Operations, variable::VariableValue};
use context::Context;

use super::Operation;

#[derive(Debug, Clone)]
pub enum Event {
    Call(Operation),
    Assignment(String, Operation),
}

impl Event {
    pub fn process<'a, 'b: 'a>(&mut self, context: &'a mut context::Runtime<'b>, operations: &Operations) -> Option<VariableValue> {
        match self {
            Self::Call(event) => event.process(context, operations),
            Self::Assignment(variable, event) => {
                let Some(return_value) = event.process(context, operations) else {
                    panic!("error: no value returned from event {event:?}"); // this case should be handled in compiletime
                };
                context.update_variable(variable, return_value);
                None
            }
        }
    }
}
