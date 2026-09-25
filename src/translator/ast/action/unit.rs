use super::*;

#[derive(Debug, Clone)]
pub enum Unit {
    Frame(Range),
    Second(Range),
    Millisecond(Range),
}

impl AstBuilder {
    pub fn get_unit(&self, node: &Node) -> Unit {
        self.expect_node_kind(node, "time_unit");
        match node.field_name_for_child(0).unwrap() {
            "frame" => Unit::Frame(Range::from(node)),
            "second" => Unit::Second(Range::from(node)),
            "millisecond" => Unit::Millisecond(Range::from(node)),
            x => panic!("error: unexpect time unit `{x}`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::macros::*;

    fn get_trigger_unit(source_code: &str) -> Unit {
        let AstNode::Action(a) = &ast!(source_code).nodes[0] else { panic!() };
        a.trigger.unit.clone()
    }

    #[test]
    fn test_unit() {
        // frame
        let unit = get_trigger_unit("at 10 frame stop;");
        assert!(matches!(unit, Unit::Frame(_)));
        let unit = get_trigger_unit("at 10 frames stop;");
        assert!(matches!(unit, Unit::Frame(_)));
        // second
        let unit = get_trigger_unit("at 10 second stop;");
        assert!(matches!(unit, Unit::Second(_)));
        let unit = get_trigger_unit("at 10 seconds stop;");
        assert!(matches!(unit, Unit::Second(_)));
        let unit = get_trigger_unit("at 10 s stop;");
        assert!(matches!(unit, Unit::Second(_)));
        // millisecond
        let unit = get_trigger_unit("at 10 millisecond stop;");
        assert!(matches!(unit, Unit::Millisecond(_)));
        let unit = get_trigger_unit("at 10 milliseconds stop;");
        assert!(matches!(unit, Unit::Millisecond(_)));
        let unit = get_trigger_unit("at 10 ms stop;");
        assert!(matches!(unit, Unit::Millisecond(_)));
    }
}
