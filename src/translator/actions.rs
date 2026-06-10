use tree_sitter::Node;

use super::{get_all_children, get_children, Translator};
use crate::{action::{Action, TimeUnit, Trigger}, event::Event, translator::error::CompilationError, variable::Variable};

impl Translator {
    pub fn get_action_definition(&mut self, node: &Node) -> Result<(), CompilationError> {
        let mut children = get_children(node);
        let label = if children.len() == 3 {
            let label_node = children.remove(0);
            self.node_to_string(&label_node)
        } else {
            String::new()
        };
        let trigger = self.get_action_trigger(&children[0]);
        let events = self.get_action_events(&children[1])?;
        if !trigger.is_enabled() && label.is_empty() {
            return Ok(());     // This action does not have to be processed, since there is no way to activate it
        }
        // let a = Action::new(label, events, trigger); // FIXME this is commented out
        self.actions.push(a);
        Ok(())
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

    fn get_action_events(&mut self, node: &Node) -> Result<Vec<Event>, CompilationError> {
        self.expect_node_kind(node, "events");
        let mut events = vec![];
        let children = get_children(node);
        if children.len() == 1 {
            events.push(self.get_event(&children[0])?);
            return Ok(events);
        }
        for e in children {
            events.push(self.get_event(&e)?);
        }
        Ok(events)
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
