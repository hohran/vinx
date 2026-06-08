use tree_sitter::Node;

use crate::translator::ast::AstBuilder;

use super::Sequence;

pub enum Time {
    Variable(String),
    Number(i64),
}

pub enum Unit {
    Frame,
    Second,
    Millisecond,
}

pub struct Trigger {
    pub onetime: bool,
    pub active: bool,
    pub time: Time,
    pub unit: Unit,
}

pub struct Action {
    pub label: Option<String>,
    pub trigger: Trigger,
    pub events: Vec<Sequence>,
}

impl AstBuilder {
    pub fn get_action(&self, node: &Node) -> Action {
        self.expect_node_kind(node, "action");
        let label = node.child_by_field_name("label").map(|n| self.get_string(&n));
        let trigger = self.get_trigger(&node.child_by_field_name("trigger").unwrap());
        let events = self.get_events(&node.child_by_field_name("events").unwrap());
        Action { label, trigger, events }
    }

    pub fn get_trigger(&self, node: &Node) -> Trigger {
        self.expect_node_kind(node, "trigger");
        let active = node.child_by_field_name("deactivated").is_some();
        let onetime = self.get_repeat_quantifier(&node.child_by_field_name("onetime").unwrap());
        let time = node.child_by_field_name("step").map_or(Time::Number(1), |n| self.get_time(&n));
        let unit = self.get_unit(&node.child_by_field_name("unit").unwrap());
        Trigger { onetime, active, time, unit }
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
            "frame" => Unit::Frame,
            "second" => Unit::Second,
            "millisecond" => Unit::Millisecond,
            x => panic!("error: unexpect time unit `{x}`"),
        }
    }
    
    pub fn get_time(&self, node: &Node) -> Time {
        match node.kind() {
            "number" => Time::Number(self.get_number(node)),
            "variable" => Time::Variable(self.get_variable(node)),
            x => panic!("error: unexpected node kind for time {node:?}: {x}"),
        }
    }

    pub fn get_events(&self, node: &Node) -> Vec<Sequence> {
        self.expect_node_kind(node, "events");
        let mut events = vec![];
        for event in node.children(&mut node.walk()) {
            match event.kind() {
                "sequence" => events.push(self.get_sequence(&event)),
                "comment" | "{" | "}" | ";" => {}
                x => panic!("error: unexpected node kind for event: `{x}")
            }
        }
        events
    }
}
