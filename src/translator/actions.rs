use std::process::exit;

use colorized::Color;
use tree_sitter::Node;

use crate::{action::{Action, TimeUnit, Trigger}, context::Context, event::Event, translator::{SequenceValue, Word, get_all_children, get_children, sequence::Sequence, translator::{Kind, Translator}}, variable::Variable};

impl Translator {
    pub fn get_action_definition(&mut self, node: &Node) {
        let mut children = get_children(node);
        let label = if children.len() == 3 {
            let label_node = children.remove(0);
            self.node_to_string(&label_node)
        } else {
            String::new()
        };
        let trigger = self.get_action_trigger(&children[0]);
        let events = self.get_action_events(&children[1]);
        if !trigger.is_enabled() && label.is_empty() {
            return;     // This action does not have to be processed, since there is no way to activate it
        }
        let a = Action::new(&label, events, trigger);
        self.actions.push(a);
    }

    fn get_action_trigger(&self, node: &Node) -> Trigger {
        let mut i = 0;
        let children = get_all_children(node);  // need to keep the ! symbol
        let active = self.get_action_active(&children[i], &mut i);
        let onetime = !self.get_action_repeats(&children[i], &mut i);
        let (t,unit) = self.get_action_trigger_time(&children, &mut i);
        let mut t = Trigger::new(t, unit, onetime);
        if !active {
            t.disable();
        }
        t
    }

    fn get_action_events(&mut self, node: &Node) -> Vec<Event> {
        node.expect_kind("events", self);
        let mut events = vec![];
        let children = get_children(node);
        if children.len() == 1 {
            events.push(self.sequence_to_event(&children[0]));
            return events;
        }
        for e in children {
            events.push(self.sequence_to_event(&e));
        }
        events
    }

    fn sequence_to_event(&mut self, node: &Node) -> Event {
        assert!(node.kind() == "sequence", "unexpected type of node {}", node.kind());
        let mut params = vec![];
        let mut seq = Sequence::new();
        for n in get_children(node) {
            match n.kind() {
                "keyword" => {
                    seq.push(Word::Keyword(self.text(&n).to_string()));
                }
                "value" => {
                    let val = self.get_atomic_value(&n);
                    if let Some(name) = self.get_variable_name(&n) {
                        params.push(Variable::new(name, val.get_type()));
                    } else {
                        params.push(Variable::new_static(val.clone()));
                    }
                    seq.push(Word::Type(val.get_type()));
                }
                x => panic!("unexpected type in sequence: {x}")
            }
        }
        let Some(sv) = self.action_decision_automaton.run(seq.get()) else {
            eprintln!("{} invalid sequence: {seq}", "error:".color(colorized::Colors::RedFg));
            eprintln!("{}:", self.file_manager.current_file().expect("error: could not retrieve file"));
            eprintln!(" line {}: {}", node.start_position().row+1, self.text(node));
            exit(1);
        };
        if let SequenceValue::Operation(id) = sv {
            self.operations[id].instantiate(params, &mut Context::empty(), &self.operations, &self.structures, &mut self.globals)
        } else {
            panic!("unexpected sequence value: {:?}", sv);
        }
    }

    fn get_action_active(&self, node: &Node, i: &mut usize) -> bool {
        if self.text(node) != "!" {
            return true;
        }
        *i += 1;
        return false;
    }

    fn get_action_repeats(&self, node: &Node, i: &mut usize) -> bool {
        *i += 1;
        self.text(node) == "every"
    }

    fn get_action_trigger_time(&self, children: &[Node], i: &mut usize) -> (Variable, TimeUnit) {
        let node = children[*i];
        let trigger_time = match node.kind() {
            "number" => {
                *i += 1;
                let val = self.node_to_int(&node);
                if val < 0 {
                    panic!("error: negative trigger time {val}");
                }
                Variable::new_static(crate::variable::VariableValue::Int(val))
            },
            "variable" => {
                *i += 1;
                let name = self.text(&node);
                Variable::new(name, crate::variable::VariableType::Int)
            },
            _ => Variable::new_static(crate::variable::VariableValue::Int(1)),
        };
        let unit = match self.text(&children[*i]) {
            "frame" | "frames" => TimeUnit::Frame,
            "s" | "second" | "seconds" => panic!("error: seconds not implemented as a unit time"),
            "ms" | "millisecond" | "milliseconds" => panic!("error: seconds not implemented as a unit time"),
            _ => panic!("unexpected time unit {}", self.text(&children[*i]))
        };
        return (trigger_time, unit)
    }
}
