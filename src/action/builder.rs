use crate::event::Event;

use super::*;

pub struct ActionBuilder {
    pub name: Option<String>,
    pub active: bool,
    pub to_activate: Option<Timestamp>,
    pub events: Option<Vec<Event>>,
    pub onetime: bool
}

impl ActionBuilder {
    pub fn new() -> Self {
        ActionBuilder { name: None, active: true, to_activate: None, events: None, onetime: false }
    }

    pub fn with_events(mut self, events: Vec<Event>) -> Self {
        self.events = Some(events);
        self
    }

    pub fn named(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn onetime(mut self, onetime: bool) -> Self {
        self.onetime = onetime;
        self
    }

    pub fn activated_at(mut self, ts: Timestamp) -> Self {
        self.to_activate = Some(ts);
        self
    }

    pub fn build(self) -> Action {
        let acc = match &self.to_activate {
            Some(Timestamp::Frame(_)) => Timestamp::Frame(0),
            Some(Timestamp::Millis(_)) => Timestamp::Millis(0),
            None => panic!("to_activate must be set before building"),
        };
        Action::new(
            self.name.expect("Action must have a name"),
            self.active,
            self.to_activate.expect("Action must have a to_activate timestamp"),
            acc,
            self.events.unwrap_or_else(|| Vec::new()),
            self.onetime
        )
    }
}
