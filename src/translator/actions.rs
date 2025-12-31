use tree_sitter::Node;

use crate::{action::{Action, Timestamp}, child, event::Event, translator::{SequenceValue, Word, seq_to_str, translator::InnerTranslator}, variable::{Variable, stack::VariableMap}};


impl<'a> InnerTranslator<'a> {
    pub fn get_action_definition(&mut self, node: &Node) {
        let mut i = 0;
        let label = self.get_action_label(node,&mut i);
        let (active,onetime,timestamp,acc) = self.get_action_trigger(&child!(node[i]));
        let events = self.get_action_events(&child!(node[i+1]));
        if !active && label.is_empty() {
            return;     // This action does not have to be processed, since there is no way to activate it
        }
        let a = Action::new(label, active, timestamp, acc, events, onetime);
        self.actions.push(a);
    }

    fn get_action_trigger(&self, node: &Node) -> (bool, bool, Timestamp, Timestamp) {
        let mut i = 0;
        let active = self.get_action_active(&child!(node[i]), &mut i);
        let onetime = !self.get_action_repeats(&child!(node[i]), &mut i);
        let (timestamp,acc) = self.get_action_timestamp_and_acc(&node, &mut i);
        (active,onetime,timestamp,acc)
    }

    fn get_action_label(&self, node: &Node, i: &mut usize) -> String {
        let n = child!(node[*i]);
        if n.kind() == "string" {
            *i += 1;
            self.node_to_string(&n)
        } else {
            "".to_string()
        }
    }

    fn get_action_events(&self, node: &Node) -> Vec<Event> {
        assert!(node.kind() == "events", "unexpected type of node {}", node.kind());
        let mut events = vec![];
        if node.child_count() == 1 {
            events.push(self.sequence_to_event(&child!(node[0])));
            return events;
        }
        for e in node.named_children(&mut self.cursor.clone()) {
            events.push(self.sequence_to_event(&e));
        }
        events
    }

    fn sequence_to_event(&self, node: &Node) -> Event {
        assert!(node.kind() == "sequence", "unexpected type of node {}", node.kind());
        let mut params = vec![];
        let mut seq = vec![];
        for n in node.children(&mut self.cursor.clone()) {
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
        let sv = self.action_decision_automaton.run(&seq).expect(&format!("error: invalid sequence: {}", seq_to_str(&seq)));
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

    fn get_action_timestamp_and_acc(&self, node: &Node, i: &mut usize) -> (Timestamp, Timestamp) {
        let q_node = child!(node[*i]);
        let quantifier = if q_node.kind() == "number" { *i += 1; self.node_to_int(&q_node) } else { 1 };
        match self.text(&child!(node[*i])) {
            "frame" | "frames" => (Timestamp::Frame(quantifier),Timestamp::Frame(0)),
            "s" | "second" | "seconds" => (Timestamp::Millis(quantifier*1000),Timestamp::Millis(0)),
            "ms" | "millisecond" | "milliseconds" => (Timestamp::Millis(quantifier),Timestamp::Millis(0)),
            _ => panic!("unexpected time unit {}", self.text(&child!(node[*i])))
        }
    }
}
