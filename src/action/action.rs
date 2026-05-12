use super::{ActionHandle, Trigger};
use crate::{context::Context, event::{Event, Operations}, variable::Stack};

/// Action is a set of events that triggers at specific timestamps.
/// In vinx, actions are either periodical: `every 10 frames { ... }`, or onetime: `at 42 frames { ... }`.
/// These types are distinguished in the Accumulator.
/// Actions can potentially be named and based on this name enabled / disabled.
#[derive(Debug)]
pub struct Action {
    name: Option<String>,
    events: Vec<Event>,
    trigger: Trigger,
}

impl Action {
    pub fn new(name: &str, events: Vec<Event>, trigger: Trigger) -> Self {
        let name = if name == "" { None } else { Some(name.to_string()) };
        Self { name, events, trigger }
    }

    pub fn is_enabled(&self) -> bool {
        self.trigger.is_enabled()
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_str())
    }

    pub fn enable(&mut self) {
        self.trigger.enable();
    }

    pub fn disable(&mut self) {
        self.trigger.disable();
    }

    /// Tell this action, that there is a new frame.
    /// This function is automatically ignored for disabled functions.
    pub fn step(&mut self) {
        self.trigger.step();
    }

    /// Try to trigger this action
    pub fn trigger(&mut self, context: &mut Context, stack: &mut Stack, operations: &Operations, action_handles: &mut Vec<ActionHandle>) {
        while self.trigger.activate(stack) {
            for event in &mut self.events {
                event.process(context, stack, action_handles, operations);
            }
        }
    }
}
