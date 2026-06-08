use image::Rgb;
use tree_sitter::Node;

use super::{Translator, CompilationError, get_children};
use crate::variable::{Direction, Effect, Variable, VariableValue};

impl Translator {
    /// Get the corresponding source code of the node.
    pub fn text(&self, node: &Node) -> &str {
        let range = node.range();
        let text = self.file_manager.current_file_contents();
        &text[range.start_byte..range.end_byte]
    }

    /// Get variable name either from a "value" or "variable" node.
    pub fn get_variable_name(&self, node: &Node) -> Option<&str> {
        if node.kind() == "value" {
            if node.child(0).unwrap().kind() != "variable" {
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

    /// Get value of a sequence given by `node`.
    ///  * `top [1,2,3]` -> 1
    pub fn get_sequence_value(&mut self, node: &Node) -> Result<VariableValue, CompilationError> {
        self.expect_node_kind(node, "sequence");
        let (seq,params) = self.get_sequence_with_params(node);
        match self.automaton.run(seq.get()) {
            Some(sv) => Ok(sv.into_value(params, &self.operations, &self.structures, &mut self.globals)),
            None => Err(CompilationError::UnknownSequence(seq, self.get_location(node)))
        }
    }

    /// Transform "value" node into VariableValue.
    ///  * 1     -> VariableValue::Int(1)
    ///  * (1,1) -> VariableValue::Pos(1,1)
    ///  * red   -> VariableValue::Color(*red*)
    pub fn get_atomic_value(&self, node: &Node) -> VariableValue {
        self.expect_node_kind(node, "value");
        let val = node.child(0).unwrap();
        match val.kind() {
            "position" => self.pos_to_val(&val),
            "color" => self.color_to_val(&val),
            "effect" => self.effect_to_val(&val),
            "variable" => {
                let var_name = self.text(&val);
                let val = self.globals.get_variable(var_name).expect(&format!("error: unknown variable {}", self.text(&val))).clone();
                val
            }
            "direction" => self.direction_to_val(&val),
            "number" => VariableValue::Int(self.node_to_int(&val) as i32),
            "string" => VariableValue::String(self.node_to_string(&val)),
            "vector" => {
                let mut v = vec![];
                for elem in get_children(&val) {
                    v.push(self.get_variable(&elem));
                }
                VariableValue::Vec(v)
            }
            _ => {
                panic!("unknown value type {}", val.kind());
            }
        }
    }

    /// Transform a "value" node into a variable:
    ///   * static variable for literals (e.g., red, 1, ...)
    ///   * otherwise named variable
    fn get_variable(&self, node: &Node) -> Variable {
        self.expect_node_kind(node, "value");
        if node.child(0).unwrap().kind() == "variable" {
            let name = self.text(&node);
            match self.globals.get_variable(name) {
                Some(val) => Variable::new(name, val.get_type()),
                None => panic!("error: unknown variable {name}"),
            }
        } else {
            self.get_atomic_value(node).to_var()
        }
    }

    /*****************************************/
    /*** SPECIFIC NODE TO VALUE CONVERTERS ***/
    /*****************************************/

    fn pos_to_val(&self,node: &Node) -> VariableValue {
        self.expect_node_kind(node, "position");
        let x = node.named_child(0).expect("expected position to have 2 child nodes");
        let y = node.named_child(1).expect("expected position to have 2 child nodes");

        VariableValue::Pos(self.node_to_int(&x), self.node_to_int(&y))
    }

    fn direction_to_val(&self, node: &Node) -> VariableValue {
        self.expect_node_kind(node, "direction");
        match self.text(&node) {
            "left" => VariableValue::Direction(Direction::Left),
            "right" => VariableValue::Direction(Direction::Right),
            "up" => VariableValue::Direction(Direction::Up),
            "down" => VariableValue::Direction(Direction::Down),
            x => panic!("unknown direction type {x}")
        }
    }

    fn effect_to_val(&self, node: &Node) -> VariableValue {
        self.expect_node_kind(node, "effect");
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
        let child = node.child(0).unwrap();
        if child.kind() == "color_name" {
            self.color_name_to_val(node)
        } else {
            self.color_code_to_val(node)
        }
    }

    fn color_name_to_val(&self, node: &Node) -> VariableValue {
        match self.text(node) {
            "red" => VariableValue::Color(Rgb([255,0,0])),
            "green" => VariableValue::Color(Rgb([0,255,0])),
            "blue" => VariableValue::Color(Rgb([0,0,255])),
            "yellow" => VariableValue::Color(Rgb([255,225,53])),
            "black" => VariableValue::Color(Rgb([0,0,0])),
            "white" => VariableValue::Color(Rgb([255,255,255])),
            "orange" => VariableValue::Color(Rgb([255,165,0])),
            "pink" => VariableValue::Color(Rgb([255,192,203])),
            "purple" => VariableValue::Color(Rgb([128,0,128])),
            "brown" => VariableValue::Color(Rgb([165,42,42])),
            "cyan" => VariableValue::Color(Rgb([0,255,255])),
            x => {
                panic!("error: unknown color name {}",x);
            }
        }
    }

    fn color_code_to_val(&self, node: &Node) -> VariableValue {
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
        VariableValue::Color(Rgb([r,g,b]))
    }

    pub fn node_to_int(&self, node: &Node) -> i32 {
        self.text(&node).parse().expect(&format!("error reading number value of node {}",node.to_string()))
    }

    pub fn node_to_string(&self, node: &Node) -> String {
        self.expect_node_kind(node, "string");
        let str = self.text(node);
        str[1..str.len()-1].to_string()
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

