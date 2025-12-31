use rsframe::vfx::video::Pixel;

use super::{types::VariableType, Variable};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableValue {
    Any(usize),  // helper type for translator
    Int(i32),   // maybe change to usize
    Pos(usize,usize),
    String(String),
    // LeftRightPos(usize),
    // UpDownPos(usize),
    Color(Pixel),
    Effect(Effect),
    Direction(Direction),
    /// Type for user defined structures
    Component(usize),
    Vec(Vec<Variable>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl ToString for Direction {
    fn to_string(&self) -> String {
        match self {
            Direction::Left => { "left".to_string() }
            Direction::Right => { "right".to_string() }
            Direction::Up => { "up".to_string() }
            Direction::Down => { "down".to_string() }
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Effect {
    Blur,
    Random,
    Inverse,
}

impl ToString for Effect {
    fn to_string(&self) -> String {
        match self {
            Self::Blur => { "blur".to_string() }
            Self::Random => { "random".to_string() }
            Self::Inverse => { "inverse".to_string() }
        }
    }
}

impl VariableValue {
    pub fn type_check(&self, v2: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(v2)
    }

    pub fn empty() -> Self {
        VariableValue::Any(0)
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Int(i) => { format!("{i}") }
            Self::Pos(x, y) => { format!("({x},{y})") }
            Self::String(s) => { format!("\"{s}\"") }
            // Self::UpDownPos(i) => { format!("{i}") }
            // Self::LeftRightPos(i) => { format!("{i}") }
            Self::Color(p) => { format!("{{{},{},{}}}",p.r,p.g,p.b) }
            Self::Effect(e) => { e.to_string() }
            Self::Direction(d) => { d.to_string() }
            Self::Component(i) => { format!("component({i})") }
            Self::Vec(v) => { let vs: Vec<String> = v.iter().map(|vv| vv.to_string()).collect(); format!("[{}]",vs.join(",")) }
            Self::Any(i) => { format!("Any({i})") }
        }
    }

    pub fn get_type(&self) -> VariableType {
        match self {
            Self::Int(_) => VariableType::Int,
            Self::Pos(_, _) => VariableType::Pos,
            Self::Color(_) => VariableType::Color,
            Self::String(_) => VariableType::String,
            Self::Effect(_) => VariableType::Effect,
            Self::Direction(_) => VariableType::Direction,
            Self::Component(i) => VariableType::Component(*i),
            Self::Vec(v) => {
                if v.len() == 0 {
                    panic!("cannot determine type of empty vector");
                }
                let t = v[0].get_type();
                for vv in v.iter().skip(1) {
                    if vv.get_type() != t {
                        panic!("vector contains mixed types");
                    }
                }
                VariableType::Vec(Box::new(t.clone()))
            }
            Self::Any(i) => VariableType::Any(*i),
        }
    }

    pub fn to_var(&self) -> Variable {
        Variable::new_static(self.clone())
    }
}
