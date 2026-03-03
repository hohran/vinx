use std::fmt::Display;

use rsframe::vfx::video::Pixel;

use crate::variable::stack::{Stack, VariableMap};

use super::{types::VariableType, Variable};

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Coordinate {
    x_rel: f32,
    x_static: i32,
    y_rel: f32,
    y_static: i32,
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x_str = if self.x_rel != 0f32 {
            if self.x_static != 0 {
                format!("{:.2} + {}", self.x_rel, self.x_static)
            } else {
                format!("{:.2}", self.x_rel)
            }
        } else {
            format!("{}", self.x_static)
        };
        let y_str = if self.y_rel != 0f32 {
            if self.y_static != 0 {
                format!("{:.2} + {}", self.y_rel, self.y_static)
            } else {
                format!("{:.2}", self.y_rel)
            }
        } else {
            format!("{}", self.y_static)
        };
        write!(f, "({},{})", x_str, y_str)
    }
}

impl Coordinate {
    pub fn new(x_rel: f32, x_static: i32, y_rel: f32, y_static: i32) -> Self {
        Self { x_rel, x_static, y_rel, y_static }
    }

    pub fn move_by(&mut self, other: &Self) {
        self.x_rel += other.x_rel;
        self.x_static += other.x_static;
        self.y_rel += other.y_rel;
        self.y_static += other.y_static;
    }

    pub fn transposed(&self) -> Self {
        Self { x_rel: self.y_rel, x_static: self.y_static, y_rel: self.x_rel, y_static: self.x_static }
    }

    pub fn get_x(&self, width: usize) -> i32 {
        let x_f = width as f32 * self.x_rel;
        if x_f > i32::MAX as f32 {
            return i32::MAX
        }
        let x: i32 = x_f as i32;
        x.saturating_add(self.x_static)
    }

    pub fn get_y(&self, height: usize) -> i32 {
        let y_f = height as f32 * self.y_rel;
        if y_f > i32::MAX as f32 {
            return i32::MAX
        }
        let y: i32 = y_f as i32;
        y.saturating_add(self.y_static)
    }
}

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

    pub fn populate_stack(&self, stack: &mut Stack) {
        for (n,v) in &self.members {
            stack.add_variable(n.clone(), v.clone());
        }
        // stack.push_layer_with(self.members.clone()); alternative ?
    }

    pub fn update(&mut self, stack: &mut Stack) {
        let member_names: Vec<String> = self.members.iter().map(|(n,_)| n.clone()).collect();
        for n in member_names {
            let v = stack.get_variable(&n).unwrap().clone();
            self.members.insert(n, v);
        }
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
    Color(Pixel),
    Effect(Effect),
    Direction(Direction),
    /// Type for user defined structures
    Structure(Structure),
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
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
            Self::Pos(x,y) => { format!("({x},{y})") }
            Self::String(s) => { format!("\"{s}\"") }
            // Self::UpDownPos(i) => { format!("{i}") }
            // Self::LeftRightPos(i) => { format!("{i}") }
            Self::Color(p) => { format!("{{{},{},{}}}",p.r,p.g,p.b) }
            Self::Effect(e) => { e.to_string() }
            Self::Direction(d) => { d.to_string() }
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
            Self::Structure(s) => VariableType::Component(s.id),
            Self::SelfReference => VariableType::SelfReference,
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

    pub fn into_color(&self) -> Pixel {
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
}
