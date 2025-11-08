use rsframe::vfx::video::Pixel;

use super::types::VariableType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariableValue {
    Any(usize),  // helper type for translator
    Label(String),   // helper type for translator
    Int(i32),   // maybe change to usize
    Pos(usize,usize),
    // LeftRightPos(usize),
    // UpDownPos(usize),
    Color(Pixel),
    Direction(Direction),
    /// Type for user defined structures
    Component(usize),
    Vec(Vec<VariableValue>),
}



#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl VariableValue {
    pub fn type_check(&self, v2: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(v2)
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Int(i) => { format!("{i}") }
            Self::Pos(x, y) => { format!("({x},{y})") }
            // Self::UpDownPos(i) => { format!("{i}") }
            // Self::LeftRightPos(i) => { format!("{i}") }
            Self::Color(p) => { format!("{{{},{},{}}}",p.r,p.g,p.b) }
            Self::Direction(d) => {
                match d {
                    Direction::Left => { "left".to_string() }
                    Direction::Right => { "right".to_string() }
                    Direction::Up => { "up".to_string() }
                    Direction::Down => { "down".to_string() }
                }
            }
            Self::Component(i) => { format!("component({i})") }
            Self::Vec(v) => { let vs: Vec<String> = v.iter().map(|vv| vv.to_string()).collect(); format!("[{}]",vs.join(",")) }
            Self::Label(_) => { panic!("error: label is only a helper value"); }
            Self::Any(_) => { panic!("error: any is only a helper value"); }
        }
    }

    pub fn get_type(&self) -> VariableType {
        match self {
            Self::Int(_) => VariableType::Int,
            Self::Pos(_, _) => VariableType::Pos,
            Self::Color(_) => VariableType::Color,
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
                VariableType::Vec(Box::new(t))
            }
            Self::Label(_) => VariableType::Label,
            Self::Any(i) => VariableType::Any(*i),
        }
    }
}
