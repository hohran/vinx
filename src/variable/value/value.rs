use std::fmt::Display;

use crate::{variable::{Variable, VariableType}, video::Frame};

use super::{Color,Effect,Direction,Structure};

/// Values of variables
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableValue {
    Any(usize),
    Int(i32),
    Pos(i32, i32),
    String(String),
    Color(Color),
    Effect(Effect),
    Direction(Direction),
    Structure(Structure),
    Image(Frame),
    // This exists only as a default value of respective VariableType
    // It should not be directly used, outside of method parsing
    SelfReference,
    Vec(Vec<Variable>),
}

impl VariableValue {
    /// Build a placeholder variable value
    pub fn placeholder() -> Self {
        VariableValue::Any(0)
    }

    /// Check if the type of this value is compatible with the other one.
    pub fn is_assignable_to(&self, other: &Self) -> bool {
        self.get_type().is_assignable_to(&other.get_type())
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
                VariableType::Vec(Box::new(t.clone()))
            }
            Self::Any(i) => VariableType::Any(*i),
        }
    }

    /// Create a static variable with this value
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

impl Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any(b) => write!(f, "Any({b})"),
            Self::Int(i) => write!(f, "{i}"),
            Self::Pos(x, y) => write!(f, "({x},{y})"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Color(c) => write!(f, "{{{},{},{}}}",c.0[0],c.0[1],c.0[2]),
            Self::Effect(e) => write!(f, "{e}"),
            Self::Direction(d) => write!(f, "{d}"),
            Self::Structure(s) => write!(f, "{s}"),
            Self::Image(i) => write!(f, "image {}x{}", i.width(), i.height()),
            Self::SelfReference => write!(f, "<structure reference>"),
            Self::Vec(v) => {
                let vs: Vec<String> = v.iter()
                    .map(|e| e.to_string())
                    .collect();
                write!(f, "[{}]", vs.join(","))
            }
        }
    }
}
