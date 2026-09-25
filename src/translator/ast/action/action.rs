use super::*;

#[derive(Debug, Clone)]
pub struct Action {
    pub label: Option<String>,
    pub trigger: Trigger,
    pub events: Vec<Event>,
    pub range: Range,
}

impl Action {
    pub fn find_variable_definition(&self, name: &str) -> Range {
        for e in &self.events {
            let Event::VarDefinition(d) = e else { continue; };
            if &d.name.0 == name {
                return d.name.1.clone();
            }
        }
        panic!("could not find variable definition for `{name}`");
    }
}

impl AstBuilder {
    pub fn get_action(&self, node: &Node) -> Action {
        self.expect_node_kind(node, "action");
        let label = node.child_by_field_name("label").map(|n| self.get_string(&n));
        let trigger = self.get_trigger(&node.child_by_field_name("trigger").unwrap());
        let events = self.get_events(&node.child_by_field_name("events").unwrap());
        Action { label, trigger, events, range: Range::from(node) }
    }
}

#[cfg(test)]
mod tests {
    use crate::translator::ast::AstNode;

    use super::*;
    use super::super::macros::*;

    fn get_action(source_code: &str) -> Action {
        let AstNode::Action(a) = &ast!(source_code).nodes[0] else { panic!() };
        a.clone()
    }

    #[test]
    fn test_ast_action() {
        let a = get_action("every frame do something;");
        assert!(a.label.is_none());
        assert_eq!(a.events.len(), 1);
        let a = get_action("\"do more\" every frame { do something; do something else; }");
        assert_eq!(a.label.as_ref().unwrap(), "do more");
        assert_eq!(a.events.len(), 2);
    }
}
