use std::process::exit;

use colorized::Color;
use tree_sitter::Node;

use crate::{action::{Action, Timestamp}, event::Event, translator::{SequenceValue, Word, get_children, seq_to_str, translator::{Kind, Translator}}, variable::{Variable, stack::VariableMap}};

impl Translator {
    pub fn get_action_definition(&mut self, node: &Node) {
        let mut children = get_children(node);
        let label = if children.len() == 3 {
            let label_node = children.remove(0);
            self.node_to_string(&label_node)
        } else {
            String::new()
        };
        let (active,onetime,timestamp,acc) = self.get_action_trigger(&children[0]);
        let events = self.get_action_events(&children[1]);
        if !active && label.is_empty() {
            return;     // This action does not have to be processed, since there is no way to activate it
        }
        let a = Action::new(label, active, timestamp, acc, events, onetime);
        self.actions.push(a);
    }

    fn get_action_trigger(&self, node: &Node) -> (bool, bool, Timestamp, Timestamp) {
        let mut i = 0;
        let children = get_children(node);
        let active = self.get_action_active(&children[i], &mut i);
        let onetime = !self.get_action_repeats(&children[i], &mut i);
        let (timestamp,acc) = self.get_action_timestamp_and_acc(&children, &mut i);
        (active,onetime,timestamp,acc)
    }

    fn get_action_events(&self, node: &Node) -> Vec<Event> {
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

    fn sequence_to_event(&self, node: &Node) -> Event {
        assert!(node.kind() == "sequence", "unexpected type of node {}", node.kind());
        let mut params = vec![];
        let mut seq = vec![];
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
                ";" => {},
                x => panic!("unexpected type in sequence: {x}")
            }
        }
        // println!("{:?}",seq_to_str(&seq));
        let Some(sv) = self.action_decision_automaton.run(&seq) else {
            eprintln!("{} invalid sequence: {}", "error:".color(colorized::Colors::RedFg), seq_to_str(&seq));
            eprintln!("{}:", self.file_manager.current_file().expect("error: could not retrieve file"));
            eprintln!(" line {}: {}", node.start_position().row+1, self.text(node));
            exit(1);
        };
        if let SequenceValue::Operation(id) = sv {
                if self.is_builtin_operation(*id) {
                    Event::new(*id, params, vec![], VariableMap::new())
                } else {
                    self.operations.get(id).expect(&format!("error: unknown operation {id}")).instantiate(params)
                }
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

    fn get_action_timestamp_and_acc(&self, children: &[Node], i: &mut usize) -> (Timestamp, Timestamp) {
        let q_node = children[*i];
        let quantifier = if q_node.kind() == "number" { *i += 1; self.node_to_int(&q_node) } else { 1 };
        match self.text(&children[*i]) {
            "frame" | "frames" => (Timestamp::Frame(quantifier),Timestamp::Frame(0)),
            "s" | "second" | "seconds" => (Timestamp::Millis(quantifier*1000),Timestamp::Millis(0)),
            "ms" | "millisecond" | "milliseconds" => (Timestamp::Millis(quantifier),Timestamp::Millis(0)),
            _ => panic!("unexpected time unit {}", self.text(&children[*i]))
        }
    }
}
