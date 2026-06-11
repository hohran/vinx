use tree_sitter::Node;

use crate::translator::ast::get_range;
use crate::translator::ast::{Assignment, AstBuilder};
use super::Range;

use super::Sequence;

#[derive(Debug, Clone)]
pub enum Time {
    Variable(String, Range),
    Number(i64, Range),
}

#[derive(Debug, Clone)]
pub enum Unit {
    Frame(Range),
    Second(Range),
    Millisecond(Range),
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub onetime: bool,
    pub active: bool,
    pub time: Time,
    pub unit: Unit,
    pub range: Range,
}

#[derive(Debug)]
pub struct Action {
    pub label: Option<String>,
    pub trigger: Trigger,
    pub events: Vec<Event>,
    pub range: Range,
}

#[derive(Debug)]
pub enum Event {
    Operation(Sequence, Range),
    Assignment(Assignment, Range),
}

impl AstBuilder {
    pub fn get_action(&self, node: &Node) -> Action {
        self.expect_node_kind(node, "action");
        let label = node.child_by_field_name("label").map(|n| self.get_string(&n));
        let trigger = self.get_trigger(&node.child_by_field_name("trigger").unwrap());
        let events = self.get_events(&node.child_by_field_name("events").unwrap());
        Action { label, trigger, events, range: get_range(node) }
    }

    pub fn get_trigger(&self, node: &Node) -> Trigger {
        self.expect_node_kind(node, "trigger");
        let active = node.child_by_field_name("deactivated").is_none();
        let onetime = self.get_repeat_quantifier(&node.child_by_field_name("onetime").unwrap());
        let time = node.child_by_field_name("step").map_or(Time::Number(1, get_range(node)), |n| self.get_time(&n));
        let unit = self.get_unit(&node.child_by_field_name("unit").unwrap());
        Trigger { onetime, active, time, unit, range: get_range(node) }
    }

    pub fn get_repeat_quantifier(&self, node: &Node) -> bool {
        self.expect_node_kind(node, "repeat_quantifier");
        match self.text(node) {
            "every" => false,
            "at" => true,
            x => panic!("error: action trigger: expected either `every` or `at`, got `{x}`"),
        }
    }

    pub fn get_unit(&self, node: &Node) -> Unit {
        self.expect_node_kind(node, "time_unit");
        match node.field_name_for_child(0).unwrap() {
            "frame" => Unit::Frame(get_range(node)),
            "second" => Unit::Second(get_range(node)),
            "millisecond" => Unit::Millisecond(get_range(node)),
            x => panic!("error: unexpect time unit `{x}`"),
        }
    }
    
    pub fn get_time(&self, node: &Node) -> Time {
        match node.kind() {
            "number" => Time::Number(self.get_number(node), get_range(node)),
            "variable" => Time::Variable(self.get_variable(node), get_range(node)),
            x => panic!("error: unexpected node kind for time {node:?}: {x}"),
        }
    }

    pub fn get_events(&self, node: &Node) -> Vec<Event> {
        self.expect_node_kind(node, "events");
        let mut events = vec![];
        for event in node.children(&mut node.walk()) {
            match event.kind() {
                "comment" | "{" | "}" | ";" => {}
                _ => events.push(self.get_event(&event)),
            }
        }
        events
    }

    pub fn get_event(&self, node: &Node) -> Event {
        self.expect_node_kind(node, "event");
        let child = node.child(0).unwrap();
        match child.kind() {
            "sequence" => Event::Operation(self.get_sequence(&child), get_range(node)),
            "assignment" => Event::Assignment(self.get_var_assignment(&child), get_range(node)),
            x => panic!("error: unexpected node kind for event: `{x}")
        }
    }
}
