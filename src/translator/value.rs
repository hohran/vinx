use rsframe::vfx::video::Pixel;
use tree_sitter::Node;

use crate::{child, translator::{SequenceValue, translator::Kind}, variable::{Variable, values::{Direction, Effect, VariableValue}}};

use super::{translator::InnerTranslator, Word};

impl<'a> InnerTranslator<'a> {
    pub fn get_variable_name(&self, node: &Node) -> Option<&str> {
        if node.kind() == "value" {
            if child!(node[0]).kind() != "variable" {
                return None;
            }
            return Some(self.text(node));
        }
        if node.kind() == "variable" {
            Some(self.text(node))
        } else {
            None
        }
    }

    pub fn get_atomic_value(&self, node: &Node) -> VariableValue {
        node.expect_kind("value");
        let val = child!(node[0]);
        match val.kind() {
            "position" => self.pos_to_val(&val),
            "color" => self.color_to_val(&val),
            "effect" => self.effect_to_val(&val),
            "variable" => self.globals.get_variable(self.text(&val)).expect(&format!("error: unknown variable {}", self.text(&val))).clone(),
            "direction" => self.direction_to_val(&val),
            "number" => VariableValue::Int(self.node_to_int(&val) as i32),
            "string" => VariableValue::String(self.node_to_string(&val)),
            "vector" => {
                let mut v = vec![];
                for elem in val.named_children(&mut self.cursor.clone()) {
                    v.push(self.get_variable(&elem));
                }
                VariableValue::Vec(v)
            }
            _ => {
                panic!("unknown value type {}", val.kind());
            }
        }
    }


    pub fn get_sequence_value(&self, node: &Node) -> VariableValue {
        node.expect_kind("sequence");
        if node.child_count() == 1 {
            return self.get_atomic_value(&child!(node[0]));
        }
        let mut seq = vec![];
        let mut params = vec![];
        for n in node.children(&mut self.cursor.clone()) {
            match n.kind() {
                "keyword" => {
                    seq.push(Word::Keyword(self.text(&n).to_string()));
                }
                "value" => {
                    let val = self.get_atomic_value(&n);
                    seq.push(Word::Type(val.get_type()));
                    params.push(val);
                }
                x => panic!("unexpected type in sequence: {x}")
            }
        }
        let sv = self.action_decision_automaton.run(&seq).expect("error: invalid sequence");
        match sv {
            SequenceValue::Component(id) => VariableValue::Component(*id),
            _ => panic!("error: unexpected sequence value {:?}", sv)
        }
    }

    fn get_variable(&self, node: &Node) -> Variable {
        node.expect_kind("value");
        let val = child!(node[0]);
        match val.kind() {
            "position" => self.pos_to_val(&val).to_var(),
            "color" => self.color_to_val(&val).to_var(),
            "effect" => self.effect_to_val(&val).to_var(),
            "variable" => {
                let name = self.text(&val);
                let var = self.globals.get_variable(name).expect(&format!("error: unknown variable {}", self.text(&val))).clone();
                Variable::new(name, var.get_type())
            }
            "direction" => self.direction_to_val(&val).to_var(),
            "number" => VariableValue::Int(self.node_to_int(&val) as i32).to_var(),
            "vector" => {
                let mut v = vec![];
                for elem in val.named_children(&mut self.cursor.clone()) {
                    v.push(self.get_variable(&elem));
                }
                VariableValue::Vec(v).to_var()
            }
            _ => {
                panic!("unknown value type {}", val.kind());
            }
        }
    }

    fn pos_to_val(&self,node: &Node) -> VariableValue {
        node.expect_kind("position");
        let x = node.named_child(0).expect("expected position to have 2 child nodes");
        let y = node.named_child(1).expect("expected position to have 2 child nodes");
        VariableValue::Pos(self.node_to_int(&x), self.node_to_int(&y))
    }

    fn direction_to_val(&self, node: &Node) -> VariableValue {
        node.expect_kind("direction");
        match self.text(&node) {
            "left" => VariableValue::Direction(Direction::Left),
            "right" => VariableValue::Direction(Direction::Right),
            "up" => VariableValue::Direction(Direction::Up),
            "down" => VariableValue::Direction(Direction::Down),
            x => panic!("unknown direction type {x}")
        }
    }

    fn effect_to_val(&self, node: &Node) -> VariableValue {
        node.expect_kind("effect");
        match self.text(&node) {
            "blurred" => VariableValue::Effect(Effect::Blur),
            "randomized" => VariableValue::Effect(Effect::Random),
            "inversed" => VariableValue::Effect(Effect::Inverse),
            x => panic!("unknown effect type {x}")
        }
    }

    fn color_to_val(&self,node: &Node) -> VariableValue {
        if node.kind() != "color" {
            panic!("expected color");
        }
        let child = child!(node[0]);
        if child.kind() == "color_name" {
            self.color_name_to_val(node)
        } else {
            self.color_code_to_val(node)
        }
    }

    fn color_name_to_val(&self, node: &Node) -> VariableValue {
        match self.text(node) {
            "red" => VariableValue::Color(Pixel::red()),
            "green" => VariableValue::Color(Pixel::green()),
            "blue" => VariableValue::Color(Pixel::blue()),
            "yellow" => VariableValue::Color(Pixel { r: 255, g: 225, b: 53 }),
            "black" => VariableValue::Color(Pixel { r: 0, g: 0, b: 0 }),
            "white" => VariableValue::Color(Pixel { r: 255, g: 255, b: 255 }),
            "orange" => VariableValue::Color(Pixel { r: 255, g: 165, b: 0 }),
            "pink" => VariableValue::Color(Pixel { r: 255, g: 192, b: 203 }),
            "purple" => VariableValue::Color(Pixel { r: 128, g: 0, b: 128 }),
            "brown" => VariableValue::Color(Pixel { r: 165, g: 42, b: 42 }),
            "cyan" => VariableValue::Color(Pixel { r: 0, g: 255, b: 255 }),
            x => {
                panic!("error: unknown color name {}",x);
            }
        }
    }

    fn color_code_to_val(&self, _node: &Node) -> VariableValue {
        todo!("error: color_code_to_val yet to be implemented");
    }

    pub fn node_to_int(&self, node: &Node) -> usize {
        self.text(&node).parse().expect(&format!("error reading number value of node {}",node.to_string()))
    }

    pub fn node_to_string(&self, node: &Node) -> String {
        node.expect_kind("string");
        let str = self.text(node);
        str[1..str.len()-1].to_string()
    }
}
