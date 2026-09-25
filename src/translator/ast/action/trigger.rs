use super::*;

#[derive(Debug, Clone)]
pub struct Trigger {
    pub onetime: bool,
    pub active: bool,
    pub time: Term,
    pub unit: Unit,
    pub range: Range,
}

impl AstBuilder {
    pub fn get_trigger(&self, node: &Node) -> Trigger {
        self.expect_node_kind(node, "trigger");
        let active = node.child_by_field_name("deactivated").is_none();
        let onetime = self.get_repeat_quantifier(&node.child_by_field_name("onetime").unwrap());
        let time = node.child_by_field_name("step").map_or(Term::Number(1), |t| self.get_value(&t));
        let unit = self.get_unit(&node.child_by_field_name("unit").unwrap());
        Trigger { onetime, active, time, unit, range: Range::from(node) }
    }

    // true => repeats
    pub fn get_repeat_quantifier(&self, node: &Node) -> bool {
        self.expect_node_kind(node, "repeat_quantifier");
        match self.text(node) {
            "every" => false,
            "at" => true,
            x => panic!("error: action trigger: expected either `every` or `at`, got `{x}`"),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::macros::*;

    fn get_trigger(source_code: &str) -> Trigger {
        let AstNode::Action(a) = &ast!(source_code).nodes[0] else { panic!() };
        a.trigger.clone()
    }

    #[test]
    fn test_ast_trigger() {
        let t = get_trigger("!every frame do something;");
        assert_eq!(t.onetime, false);
        assert_eq!(t.active, false);
        assert!(matches!(t.time, Term::Number(1)));
        assert!(matches!(t.unit, Unit::Frame(_)));
        //
        let t = get_trigger("at $x seconds do something;");
        assert_eq!(t.onetime, true);
        assert_eq!(t.active, true);
        let Term::Variable((time_var, _)) = &t.time else { panic!() };
        assert_eq!(time_var, "$x");
        assert!(matches!(t.unit, Unit::Second(_)));
        //
        let t = get_trigger("at (get random number) ms { do something; }");
        assert_eq!(t.onetime, true);
        assert_eq!(t.active, true);
        assert!(matches!(t.time, Term::Expression(_)));
        assert!(matches!(t.unit, Unit::Millisecond(_)));
    }
}
