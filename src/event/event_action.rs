use crate::{context, translator::parser::Call, variable::VariableValue};
use context::Context;

#[derive(Clone,PartialEq,Debug)]
pub enum Event {
    Call(Call),
    Assignment(String, Call),
}

impl Event {
    // pub fn process<'a, 'b: 'a>(&mut self, context: &'a mut context::Runtime<'b>, operations: &Operations) -> Option<VariableValue> {
    //     match self {
    //         Self::Call(event) => event.process(context, operations),
    //         Self::Assignment(variable, event) => {
    //             let Some(return_value) = event.process(context, operations) else {
    //                 panic!("error: no value returned from event {event:?}"); // this case should be handled in compiletime
    //             };
    //             context.update_variable(variable, return_value);
    //             None
    //         }
    //     }
    // }

    pub fn process<'a, 'b: 'a>(&mut self, context: &'a mut context::Runtime<'b>) -> Option<VariableValue> {
        match self {
            Self::Call(event) => event.evaluate_at_runtime(context),
            Self::Assignment(variable, event) => {
                let Some(return_value) = event.evaluate_at_runtime(context) else {
                    panic!("error: no value returned from event {event:?}"); // this case should be handled in compiletime
                };
                context.update_variable(variable, return_value);
                None
            }
        }
    }
}
