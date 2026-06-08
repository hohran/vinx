use tree_sitter::Node;

use super::AstBuilder;
use crate::variable::{Direction, Effect};

type Color = (u8, u8, u8); // TODO: add alpha
type Position = (i64, i64);

pub enum Value {
    Variable(String),
    Number(i64),
    Position(Position), // TODO: make into Tuple
    Color(Color),
    Effect(Effect),
    Direction(Direction),
    String(String),
    Vector(Vec<Value>),
}

impl AstBuilder {
    pub fn get_value(&self, node: &Node) -> Value {
        self.expect_node_kind(node, "value");
        let val = node.child(0).unwrap();
        match val.kind() {
            "variable" => Value::Variable(self.get_variable(&val)),
            "position" => Value::Position(self.get_position(&val)),
            "color" => Value::Color(self.get_color(&val)),
            "effect" => Value::Effect(self.get_effect(&val)),
            "direction" => Value::Direction(self.get_direction(&val)),
            "number" => Value::Number(self.get_number(&val)),
            "string" => Value::String(self.get_string(&val)),
            "vector" => Value::Vector(self.get_vector(&val)),
            x => {
                panic!("unknown value type {x}");
            }
        }
    }

    pub fn get_variable(&self, node: &Node) -> String {
        self.expect_node_kind(node, "variable");
        self.text(node).to_string()
    }

    fn get_position(&self, node: &Node) -> (i64, i64) {
        self.expect_node_kind(node, "position");
        let x = node.named_child(0).unwrap();
        let y = node.named_child(1).unwrap();
        (self.get_number(&x),self.get_number(&y))
    }

    pub fn get_number(&self, node: &Node) -> i64 {
        self.expect_node_kind(node, "number");
        self.text(&node).parse().expect(&format!("error reading number value of node {}",node.to_string()))
    }

    pub fn get_string(&self, node: &Node) -> String {
        self.expect_node_kind(node, "string");
        let str = self.text(node);
        str[1..str.len()-1].to_string()
    }

    fn get_direction(&self, node: &Node) -> Direction {
        self.expect_node_kind(node, "direction");
        match self.text(&node) {
            "left" => Direction::Left,
            "right" => Direction::Right,
            "up" => Direction::Up,
            "down" => Direction::Down,
            x => panic!("unknown direction type {x}")
        }
    }

    fn get_effect(&self, node: &Node) -> Effect {
        self.expect_node_kind(node, "effect");
        match self.text(&node) {
            "blurred" => Effect::Blur,
            "randomized" => Effect::Random,
            "inversed" => Effect::Inverse,
            x => panic!("unknown effect type {x}")
        }
    }

    fn get_color(&self,node: &Node) -> Color {
        if node.kind() != "color" {
            panic!("expected color");
        }
        let child = node.child(0).unwrap();
        if child.kind() == "color_name" {
            self.get_color_by_name(node)
        } else {
            self.get_color_by_val(node)
        }
    }

    fn get_color_by_name(&self, node: &Node) -> Color {
        match self.text(node) {
            "red" => (255,0,0),
            "green" => (0,255,0),
            "blue" => (0,0,255),
            "yellow" => (255,225,53),
            "black" => (0,0,0),
            "white" => (255,255,255),
            "orange" => (255,165,0),
            "pink" => (255,192,203),
            "purple" => (128,0,128),
            "brown" => (165,42,42),
            "cyan" => (0,255,255),
            x => {
                panic!("error: unknown color name {}",x);
            }
        }
    }

    fn get_color_by_val(&self, node: &Node) -> Color {
        let val = self.text(&node);
        let mut it = val.chars().skip(1); // skip '#'
        let mut r = 0;
        if let Some(c) = it.next() { r = c2i(c)*16; }
        if let Some(c) = it.next() { r += c2i(c); }
        let mut g = 0;
        if let Some(c) = it.next() { g = c2i(c)*16; }
        if let Some(c) = it.next() { g += c2i(c); }
        let mut b = 0;
        if let Some(c) = it.next() { b = c2i(c)*16; }
        if let Some(c) = it.next() { b += c2i(c); }
        (r,g,b)
    }

    fn get_vector(&self, node: &Node) -> Vec<Value> {
        let mut v = vec![];
        let ignored_node_kinds = ["comment", "[", "]", ","];
        for val in node.children(&mut node.walk()) {
            if ignored_node_kinds.contains(&val.kind()) {
                continue;
            }
            v.push(self.get_value(&val));
        }
        v
    }
}

/// Convert char to int (for color value resolution).
fn c2i(c: char) -> u8 {
    if c <= '9' {
        (c as u8) - ('0' as u8)
    } else if c <= 'F' {
        (c as u8) - ('A' as u8) + 10
    } else {
        (c as u8) - ('a' as u8) + 10
    }
}

// TODO: test
#[cfg(test)]
mod tests {
    use super::*;
}
