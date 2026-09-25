use super::*;

// TODO: move to top level ast folder ... it is not only for actions

#[derive(Debug, Clone)]
pub enum Event {
    Operation(Expr),
    Assignment(Assignment),
    VarDefinition(VarDefinition),
}

impl AstBuilder {
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
            "expr" => Event::Operation(self.get_expr(&child)),
            "assignment" => Event::Assignment(self.get_var_assignment(&child)),
            "var_definition" => Event::VarDefinition(self.get_var_definition(&child)),
            x => panic!("error: unexpected node kind for event: `{x}")
        }
    }
}
