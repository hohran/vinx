use crate::{context::Context, event::{Event, Operations}, variable::Stack};
use std::collections::HashMap;

#[derive(Debug,Clone)]
pub enum Timestamp {
    Frame(usize),
    Millis(usize),
}

#[derive(Debug,Clone)]
pub struct Action {
    name: String,
    active: bool,
    to_activate: Timestamp,
    time_accumulator: Timestamp,
    events: Vec<Event>,
    onetime: bool,
    activated: bool,
}

impl Action {
    pub fn new(name: String, active: bool, to_activate: Timestamp, time_accumulator: Timestamp, events: Vec<Event>, onetime: bool) -> Self {
        assert!(std::mem::discriminant(&to_activate) == std::mem::discriminant(&time_accumulator), "to_activate and time_accumulator must be of the same type");
        match &to_activate {
            Timestamp::Frame(t) => assert!(*t > 0, "to_activate must be greater than 0"),
            Timestamp::Millis(t) => assert!(*t > 0, "to_activate must be greater than 0"),
        }
        Action { name, active, to_activate, time_accumulator, events, onetime, activated: false }
    }

    pub fn is_active(&self, action_activeness: &HashMap<String,bool>) -> bool {
        *action_activeness.get(&self.name).expect("error: could not retrieve action activeness")
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn default_activeness(&self) -> bool {
        self.active
    }

    pub fn clear_accumulator(&mut self) {
        self.time_accumulator = match self.to_activate {
            Timestamp::Frame(_) => Timestamp::Frame(0),
            Timestamp::Millis(_) => Timestamp::Millis(0),
        };
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn get_events(&self) -> &Vec<Event> {
        &self.events
    }

    pub fn is_onetime(&self) -> bool {
        self.onetime
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn step(&mut self, millis: usize) {
        match &mut self.time_accumulator {
            Timestamp::Frame(f) => {
                *f += 1;
            }
            Timestamp::Millis(m) => {
                *m += millis;
            }
        }
    }

    pub fn trigger(&mut self, context: &mut Context, stack: &mut Stack, action_activeness: &mut HashMap<String,bool>, operations: &Operations ) {
        if self.onetime && self.activated { return; }
        match (&self.to_activate, &self.time_accumulator) {
            (Timestamp::Frame(i), Timestamp::Frame(acc)) => {
                if *acc >= *i {
                    self.process_events(context, stack, action_activeness, operations);
                    if self.onetime {
                        self.activated = true;
                    }
                    self.clear_accumulator();
                }   
            }
            (Timestamp::Millis(i), Timestamp::Millis(acc)) => {
                let mut tmp_acc = *acc;
                let tmp_i = *i;
                while tmp_acc >= tmp_i {
                    self.process_events(context, stack, action_activeness, operations);
                    if self.onetime {
                        self.activated = true;
                        break;
                    }
                    tmp_acc -= tmp_i;
                }
                self.time_accumulator = Timestamp::Millis(tmp_acc);
            }
            _ => panic!("to_activate and time_accumulator must be of the same type"),
        }
    }

    fn process_events(&mut self, context: &mut Context, stack: &mut Stack, action_activeness: &mut HashMap<String,bool>, operations: &Operations) {
        for event in &mut self.events {
            event.process(context, stack, action_activeness, operations);
        }
    }
}
