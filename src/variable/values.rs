use std::fmt::Display;

use image::Rgb;

use crate::{variable::stack::{Stack, VariableMap}, video::Frame};

use super::{types::VariableType, Variable};

pub type Color = Rgb<u8>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Structure {
    pub id: usize,
    members: VariableMap,
}

impl Structure {
    pub fn new(id: usize, members: VariableMap) -> Self {
        Self { id, members }
    }

    pub fn default(id: usize) -> Self {
        Self { id, members: VariableMap::new() }
    }

    pub fn copy_members(&self) -> VariableMap {
        self.members.clone()
    }

    pub fn update(&mut self, stack: &mut Stack) {
        let member_names: Vec<String> = self.members.iter().map(|(n,_)| n.clone()).collect();
        for n in member_names {
            let v = stack.get_variable(&n).unwrap().clone();
            self.members.insert(n, v);
        }
    }

    pub fn get_member(&self, name: &str) -> &VariableValue {
        self.members.get(name).expect(&format!("error: could not find member {name} in structure {}", self.id))
    }

    pub fn get_member_mut(&mut self, name: &str) -> &mut VariableValue {
        self.members.get_mut(name).expect(&format!("error: could not find member {name} in structure {}", self.id))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableValue {
    Any(usize),  // helper type for translator
    Int(i32),   // maybe change to usize
    Pos(i32, i32),
    String(String),
    // LeftRightPos(usize),
    // UpDownPos(usize),
    Color(Color),
    Effect(Effect),
    Direction(Direction),
    /// Type for user defined structures
    Structure(Structure),
    Image(Frame),
    SelfReference,
    Vec(Vec<Variable>),
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
            Direction::Up => write!(f, "up"),
            Direction::Down => write!(f, "down"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Effect {
    Blur,
    Random,
    Inverse,
}

impl Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blur => write!(f, "blur"),
            Self::Random => write!(f, "random"),
            Self::Inverse => write!(f, "inverse"),
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

    pub fn is_assignable_to(&self, other: &Self) -> bool {
        self.get_type().is_assignable_to(&other.get_type())
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Int(i) => { format!("{i}") }
            Self::Pos(x,y) => { format!("({x},{y})") }
            Self::String(s) => { format!("\"{s}\"") }
            // Self::UpDownPos(i) => { format!("{i}") }
            // Self::LeftRightPos(i) => { format!("{i}") }
            Self::Color(p) => { format!("{{{},{},{}}}",p.0[0],p.0[1],p.0[2]) }
            Self::Effect(e) => { e.to_string() }
            Self::Direction(d) => { d.to_string() }
            Self::Image(i) => { 
                let w = i.width();
                let h = i.height();
                format!("image {w}x{h}")
            }
            Self::Structure(s) => { 
                let mut map_str = String::new();
                for (n,v) in &s.members {
                    map_str += &(n.to_string() + "=>" + &v.to_string() + ", ");
                }
                format!("<struct({}) {map_str}>",s.id) }
            Self::Vec(v) => { let vs: Vec<String> = v.iter().map(|vv| vv.to_string()).collect(); format!("[{}]",vs.join(",")) }
            Self::Any(i) => { format!("Any({i})") }
            Self::SelfReference => { format!("<self reference>") }
        }
    }

    pub fn get_type(&self) -> VariableType {
        match self {
            Self::Int(_) => VariableType::Int,
            Self::Pos(_,_) => VariableType::Pos,
            Self::Color(_) => VariableType::Color,
            Self::String(_) => VariableType::String,
            Self::Effect(_) => VariableType::Effect,
            Self::Direction(_) => VariableType::Direction,
            Self::Image(_) => VariableType::Image,
            Self::Structure(s) => VariableType::Structure(s.id),
            Self::SelfReference => VariableType::SelfReference,
            Self::Vec(v) => {
                if v.len() == 0 {
                    panic!("cannot determine type of empty vector");
                }
                let t = v[0].get_type();
                // for item in v.iter().skip(1) {
                //     if item.get_type() != t {
                //         panic!("vector contains mixed types");
                //     }
                // }
                VariableType::Vec(Box::new(t.clone()))
            }
            Self::Any(i) => VariableType::Any(*i),
        }
    }

    pub fn to_var(&self) -> Variable {
        Variable::new_static(self.clone())
    }

    /** Function for conversion to type **/

    pub fn into_string(&self) -> &str {
        let Self::String(s) = self else {
            panic!();
        };
        s
    }

    pub fn into_int(&self) -> i32 {
        let Self::Int(i) = self else {
            panic!();
        };
        *i
    }

    pub fn into_pos(&self) -> (i32,i32) {
        let Self::Pos(x, y) = self else {
            panic!();
        };
        (*x,*y)
    }

    pub fn into_vec(&self) -> &Vec<Variable> {
        let Self::Vec(v) = self else {
            panic!();
        };
        v
    }

    pub fn into_direction(&self) -> Direction {
        let Self::Direction(d) = self else {
            panic!();
        };
        *d
    }

    pub fn into_color(&self) -> Color {
        let Self::Color(c) = self else {
            panic!();
        };
        *c
    }

    pub fn into_effect(&self) -> Effect {
        let Self::Effect(e) = self else {
            panic!();
        };
        *e
    }

    pub fn into_image(&self) -> &Frame {
        let Self::Image(i) = self else {
            panic!();
        };
        i
    }
}
