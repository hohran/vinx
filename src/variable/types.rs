use rsframe::vfx::video::Pixel;

use super::values::{Direction, VariableValue};

#[macro_export]
macro_rules! vtype {
    ( [ $($x:tt)+ ] ) => { VariableType::Vec(Box::new(vtype!($($x)+))) };
    ( String ) => { VariableType::String };
    ( Int ) => { VariableType::Int };
    ( Pos ) => { VariableType::Pos };
    ( Color ) => { VariableType::Color };
    ( Direction ) => { VariableType::Direction };
    ( Column ) => { VariableType::Column };
    ( Component($i:expr) ) => { VariableType::Component($i) };
    ( Any ( $i:expr ) ) => { VariableType::Any($i) };
    ( ( $($x:tt)+ ) ) => { vtype!($($x)+) };
}

#[derive(Hash,Clone, Debug, Eq)]
pub enum VariableType {
    Int,
    Pos,
    Color,
    String,
    Effect,
    Direction,
    Vec(Box<VariableType>),
    Any(usize),
    /// Type for user defined structures
    Component(usize),
    /// Error type for type inference
    None,
}

impl ToString for VariableType {
    fn to_string(&self) -> String {
        format!("{:?}",self)
    }
}

impl PartialEq for VariableType {
    fn eq(&self, other: &Self) -> bool {
        match (self,other) {
            (VariableType::Component(x),VariableType::Component(y)) => x==y,
            (VariableType::Any(_),VariableType::Any(_)) => true,
            (VariableType::Vec(x),VariableType::Vec(y)) => {
                x == y
            }
            (x,y) => std::mem::discriminant(x) == std::mem::discriminant(y),
        }
    }
}

impl VariableType {
    /// Returns whether a contains Any
    /// Int -> false
    /// Any(_) -> true
    /// [Any(_)] -> true
    pub fn is_ambiguous(&self) -> bool {
        match self {
            VariableType::Any(_) => true,
            VariableType::Vec(x) => x.is_ambiguous(),
            _ => false,
        }
    }

    /// Returns default value for given type
    /// Int -> Int(0)
    /// Pos -> Pos(0,0)
    pub fn default(&self) -> VariableValue {
        match &self {
            VariableType::Vec(x) => { VariableValue::Vec(vec![x.default().to_var()]) }
            VariableType::Int => { VariableValue::Int(0) }
            VariableType::Pos => { VariableValue::Pos(0, 0) }
            VariableType::Direction => { VariableValue::Direction(Direction::Left) }
            VariableType::Color => { VariableValue::Color(Pixel::black()) }
            VariableType::String => { VariableValue::String("".to_string()) }
            VariableType::Effect => { VariableValue::Effect(crate::variable::values::Effect::Blur) }
            VariableType::Any(x) => { VariableValue::Any(*x) }
            VariableType::Component(x) => { VariableValue::Component(*x) }
            VariableType::None => { panic!("error: tried to instantiate None type") }
        }
    }

    /// Pos + Int -> None
    /// Int + Int -> Int
    /// Any(1) + Int -> Int
    /// [Any(1)] + [[Int]] -> [[Int]]
    /// Any(1) + Any(2) -> Any(1)
    /// Any(1) + [Any(1)] -> None !!! think about it
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        match (self,other) {
            (VariableType::Vec(x), VariableType::Vec(y)) => {
                if let Some(t) = x.intersect(y) {
                    return Some(VariableType::Vec(Box::new(t)));
                }
                None
            },
            (VariableType::Any(x), t) | (t, VariableType::Any(x)) => {
                if let Some(y) = t.get_binding() {
                    if y == *x && self != other {
                        return None;
                    }
                }
                Some(t.clone())
            },
            (VariableType::None, _) | (_, VariableType::None) => None,
            _ => {
                if self == other { 
                    Some(self.clone()) 
                } else {
                    None
                }
            },
        }
    }

    /// Int in Int -> false
    /// Int in Pos -> false
    /// Int in Any(1) -> true
    /// Any(1) in Int -> false
    /// Any(1) in Any(2) -> true
    /// \[Int] in \[Any(1)] -> true
    /// \[Int] in Any(1) -> true
    pub fn is_subset_of(&self, other: &Self) -> bool {
        if let VariableType::Any(_) = other { 
            return true;
        }
        match (self,other) {
            (VariableType::Vec(a),VariableType::Vec(b)) => a.is_subset_of(b),
            _ => false
        }
    }

    /// Get binding of ambiguous types
    /// Any(1) -> 1
    /// [[Any(3)]] -> 3
    /// Int -> None
    pub fn get_binding(&self) -> Option<usize> {
        match self {
            VariableType::Any(x) => Some(*x),
            VariableType::Vec(x) => x.get_binding(),
            _ => None
        }
    }

    /// for ambiguous types: convert binding x into MAX-x
    /// this is done so that bindings from different context dont get mixed up
    pub fn with_inverted_binding(&self) -> Self {
        match self {
            VariableType::Any(x) => VariableType::Any(usize::MAX - *x),
            VariableType::Vec(x) => VariableType::Vec(Box::new(x.with_inverted_binding())),
            _ => self.clone()
        }
    }

    /// Set binding of ambigous type.
    /// If the type is not ambiguous, nothing happens
    /// Any(x) + 42 -> Any(42)
    /// [[Any(y)]] + 1 -> [[Any(1)]]
    /// Pos + 22 -> Pos
    pub fn set_binding(&mut self, x: usize) {
        match self {
            VariableType::Any(_) => *self = VariableType::Any(usize::min(x, usize::MAX-x)),
            VariableType::Vec(v) => v.set_binding(x),
            _ => {}
        }
    }

    /// If the binding number matches, switch ambiguous type for a new one
    /// Any(1).set_binding_type(1, Int) -> Int
    /// Any(1).set_binding_type(2, Int) -> Any(1)
    /// [Any(1)].set_binding_type(1, Int) -> [Int]
    pub fn set_binding_type(&mut self, binding: usize, new_type: &Self) {
        match self {
            VariableType::Any(x) => {
                if *x != binding { return; }
                *self = new_type.clone();
            }
            VariableType::Vec(v) => v.set_binding_type(binding, new_type),
            _ => {}
        }
    }

    /// strictly_matches(Any(0), Any(1)) -> false
    /// normally Any(0) == Any(1) -> true
    pub fn strictly_matches(&self, other: &Self) -> bool {
        self == other && self.get_binding() == other.get_binding()
    }

    /// Int -> 0
    /// [Pos] -> 1
    /// [[[Any(42)]]] -> 3
    pub fn get_depth(&self) -> usize {
        let mut x = self;
        let mut depth = 0;
        loop {
            match x {
                VariableType::Vec(nx) => {
                    depth += 1;
                    x = nx;
                }
                _ => {
                    return depth;
                }
            }
        }
    }

    pub fn unwrap_depth(&self, mut depth: usize) -> Self {
        let mut x = self;
        loop {
            if depth == 0 { return x.clone() }
            if let VariableType::Vec(nx) = x {
                x = nx;
                depth -= 1;
            } else {
                panic!("error: {:?} is not deep enough", self);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vtype;

    #[test]
    fn test_macro() {
        // TODO
    }

    #[test]
    fn test_is_ambiguous() {
        // TODO
    }

    #[test]
    fn test_intersect() {
        let vt1 = vtype!(Pos);
        let vt2 = vtype!(Int);
        let vt3 = vtype!(Any(1));
        let vt4 = vtype!(Any(2));
        let vt5 = vtype!([Any(1)]);
        let vt6 = vtype!([[Int]]);
        // Pos + Int -> None
        assert_eq!(vt1.intersect(&vt2), None);
        // Int + Int -> Int
        assert_eq!(vt2.intersect(&vt2), Some(vtype!(Int)));
        // Any(1) + Int -> Int
        assert_eq!(vt3.intersect(&vt2), Some(vtype!(Int)));
        // [Any(1)] + [[Int]] -> [[Int]]
        assert_eq!(vt5.intersect(&vt6), Some(vtype!([[Int]])));
        // Any(1) + Any(2) -> Any(1)
        assert_eq!(vt3.intersect(&vt4), Some(vtype!(Any(1))));
        // Any(1) + [Any(1)] -> None !!! think about it
        assert_eq!(vt3.intersect(&vt5), None);
    }

    #[test]
    fn test_strictly_matches() {
        let vt1 = vtype!(Any(1));
        let vt2 = vtype!(Any(2));
        assert!(vt1.strictly_matches(&vt1));
        assert!(!vt1.strictly_matches(&vt2));
    }

    #[test]
    fn test_get_binding() {
        let vt1 = vtype!(Int);
        let vt2 = vtype!(Any(1));
        let vt3 = vtype!([Any(2)]);
        let vt4 = vtype!([Int]);

        assert_eq!(vt1.get_binding(), None);
        assert_eq!(vt2.get_binding(), Some(1));
        assert_eq!(vt3.get_binding(), Some(2));
        assert_eq!(vt4.get_binding(), None);
    }

    #[test]
    fn test_invert_binding() {
        let vt1 = vtype!(Int);
        let vt2 = vtype!(Any(1));
        let vt3 = vtype!([Any(2)]);
        let vt4 = vtype!([Int]);

        assert_eq!(vt1.with_inverted_binding().get_binding(), None);
        assert_eq!(vt2.with_inverted_binding().get_binding(), Some(usize::MAX-1));
        assert_eq!(vt3.with_inverted_binding().get_binding(), Some(usize::MAX-2));
        assert_eq!(vt4.with_inverted_binding().get_binding(), None);
    }

    #[test]
    fn test_set_binding() {
        let mut vt1 = vtype!(Int);
        let mut vt2 = vtype!(Any(1));
        let mut vt3 = vtype!([Any(2)]);
        let mut vt4 = vtype!([Int]);

        vt1.set_binding(0);
        vt2.set_binding(0);
        vt3.set_binding(0);
        vt4.set_binding(0);

        assert_eq!(vt1.get_binding(), None);
        assert_eq!(vt2.get_binding(), Some(0));
        assert_eq!(vt3.get_binding(), Some(0));
        assert_eq!(vt4.get_binding(), None);
    }

    #[test]
    fn test_set_binding_type() {
        let mut vt1 = vtype!(Pos);
        let mut vt2 = vtype!(Any(1));
        let mut vt3 = vtype!(Any(2));
        let mut vt4 = vtype!([Any(1)]);

        vt1.set_binding_type(1, &vtype!(Int));
        vt2.set_binding_type(1, &vtype!(Int));
        vt3.set_binding_type(1, &vtype!(Int));
        vt4.set_binding_type(1, &vtype!(Int));

        assert_eq!(vt1, vtype!(Pos));
        assert_eq!(vt2, vtype!(Int));
        assert_eq!(vt3, vtype!(Any(2)));
        assert_eq!(vt3.get_binding(), Some(2));
        assert_eq!(vt4, vtype!([Int]));
    }

    #[test]
    fn test_get_depth() {
        let vt1 = vtype!(Any(0));
        let vt2 = vtype!([Any(0)]);
        let vt3 = vtype!([[Any(0)]]);
        let vt4 = vtype!([[[Any(0)]]]);
        let vt5 = vtype!([[[Any(1)]]]);
        assert_eq!(vt1.get_depth(), 0);
        assert_eq!(vt2.get_depth(), 1);
        assert_eq!(vt3.get_depth(), 2);
        assert_eq!(vt4.get_depth(), 3);
        assert_eq!(vt5.get_depth(), 3);
    }

    #[test]
    fn test_unwrap_depth() {
        let vt1 = vtype!(Int);
        let vt2 = vtype!([Int]);
        let vt3 = vtype!([[Int]]);
        let vt4 = vtype!([[[Int]]]);
        assert_eq!(vt1.unwrap_depth(0), vtype!(Int));
        assert_eq!(vt2.unwrap_depth(1), vtype!(Int));
        assert_eq!(vt3.unwrap_depth(2), vtype!(Int));
        assert_eq!(vt4.unwrap_depth(3), vtype!(Int));
    }
}
