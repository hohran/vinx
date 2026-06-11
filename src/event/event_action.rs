// TODO: This should be renamed to Event (Event should be renamed to Operation and Operation to
// OperationTemplate)

use crate::{action::ActionHandle, context::Context, event::Operations, variable::{Stack, VariableValue}};

use super::Operation;

#[derive(Debug, Clone)]
pub enum Event {
    Call(Operation),
    Assignment(String, Operation),
}

impl Event {
    pub fn process(&mut self, context: &mut Context, stack: &mut Stack, action_handles: &mut Vec<ActionHandle>, operations: &Operations) -> Option<VariableValue> {
        match self {
            Self::Call(event) => event.process(context, stack, action_handles, operations),
            Self::Assignment(variable, event) => {
                let Some(return_value) = event.process(context, stack, action_handles, operations) else {
                    panic!("error: no value returned from event {event:?}");
                };
                stack.update_variable(variable, return_value);
                None
            }
        }
    }
}
