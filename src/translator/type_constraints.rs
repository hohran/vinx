use std::{cmp::max, fmt::Display};

use crate::variable::types::VariableType;

#[derive(Debug,PartialEq, Eq, Clone, Hash)]
pub struct TypeConstraints {
    types: Vec<VariableType>,
}

impl Display for TypeConstraints {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let types_str: Vec<String> = self.types.iter().map(|x| x.to_string()).collect();
        write!(f, "[{}]", types_str.join(" "))
    }
}

impl TypeConstraints {
    pub fn new(var_count: usize) -> Self {
        let mut types = vec![];
        for i in 0..var_count {
            types.push(VariableType::Any(i));
        }
        Self { types }
    }

    pub fn _new() -> Self {
        Self { types: vec![] }
    }

    pub fn from(types: Vec<VariableType>) -> Self {
        Self { types }
    }

    pub fn cut_to(self, to: usize) -> Self {
        Self::from(self.types[..to].to_vec())
    }

    pub fn cut_from(self, from: usize) -> Self {
        Self::from(self.types[from..].to_vec())
    }

    pub fn at(&self, at: usize) -> VariableType {
        if at < self.types.len() {
            self.types[at].clone()
        } else {
            VariableType::Any(at)
        }
    }

    pub fn get_types(&self) -> &Vec<VariableType> {
        &self.types
    }

    pub fn resize_to(&mut self, new_size: usize) {
        let current_size = self.types.len();
        if new_size == current_size {
            return;
        }
        for i in current_size..new_size {
            self.types.push(VariableType::Any(i));
        }
    }

    /// Set binding numbers of all ambiguous types, so that they are ascending and starting at 0.
    /// [ Any(12), \[Any(89)], Int, Any(89) ] -> [ Any(0), \[Any(1)], Int, Any(1) ]
    pub fn refresh_bindings(&mut self) {
        let mut found_bindings = vec![];
        for vt in &mut self.types {
            if let Some(binding) = vt.get_binding() {
                if let Some(n) = found_bindings.iter().position(|b| *b == binding) {
                    vt.set_binding(n);
                } else {
                    vt.set_binding(found_bindings.len());
                    found_bindings.push(binding);
                }
            }
        }
    }

    pub fn strictly_matches(&self, other: &Self) -> bool {
        for i in 0..self.types.len() {
            if !self.types[i].strictly_matches(&other.types[i]) {
                return false;
            }
        }
        true
    }

    /// Refine all types of with given binding with new_type.
    pub fn update_binding(&mut self, binding: usize, new_type: VariableType) {
        for i in 0..self.types.len() {
            let vt = &mut self.types[i];
            vt.set_binding_type(binding, &new_type);
        }
    }

    fn intersect_at(&mut self, other: &Self, at: usize) -> bool {
        let self_at = &self.types[at];
        let other_at = &other.types[at];
        if !self_at.is_ambiguous() && !other_at.is_ambiguous() {
            if self_at != other_at {
                return false;
            }
        }
        // this means that self_at can be refined by other_at
        if other_at.is_subset_of(&self.types[at]) {
            // disallow that Any(0) is assignable with [Any(0)]
            if self_at.get_binding() == other_at.with_inverted_binding().get_binding() && self_at.get_depth() != other_at.get_depth() {
                return false;
            }
            let depth = self_at.get_depth();     // since self in other: other has always lower depth
            self.update_binding(
                self.types[at].get_binding()                   // binding of current type
                .expect("error: expected to be ambiguous"),  // if it is a superset, it should always be ambiguous (it will have a binding number)
                other.types[at]                  // for the current type
                .unwrap_depth(depth)            // this is the part to update with
                .with_inverted_binding()        // invert binding so that it does not get mixed up with currently used bindings (if it is ambiguous)
            );
        }
        true
    }

    pub fn intersect(mut self, mut other: Self) -> Option<Self> {
        let len = max(self.types.len(), other.types.len());
        self.resize_to(len);
        other.resize_to(len);
        loop {
            let mut self_c = self.clone();
            let mut other_c = other.clone();
            for i in 0..len {
                if !self_c.intersect_at(&other, i) {
                    return None;
                }
                if !other_c.intersect_at(&self, i) {
                    return None;
                }
            }
            self_c.refresh_bindings();
            other_c.refresh_bindings();
            if self_c.strictly_matches(&self) && other_c.strictly_matches(&other) {
                break;
            } else {
                self = self_c;
                other = other_c;
            }
        }
        let mut out = Self::new(self.types.len());
        for i in 0..self.types.len() {
            if let Some(vt) = self.types[i].intersect(&other.types[i]) {
                out.types[i] = vt;
            } else {
                return None;
            }
        }
        Some(out)
    }

    /// Intersects variable with var_id.
    /// Returns if it is intersectable with var_type
    pub fn intersect_var(&mut self, var_type: &VariableType, var_id: usize) -> bool {
        self.resize_to(var_id+1);
        let var = &self.types[var_id];
        if let Some(prod) = var.intersect(var_type) {
            self.types[var_id] = prod;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vtype;

    macro_rules! constaint {
        ( $($x:tt)+ ) => {
            TypeConstraints::from(([$(vtype!($x)),+]).to_vec())
        };
    }

    #[test]
    fn test_macro() {
        let tc = constaint![(Any(0)) (Any(1))];
        assert_eq!(tc, TypeConstraints::new(2));
        assert_eq!(tc.types[0].get_binding(), Some(0));
        assert_eq!(tc.types[1].get_binding(), Some(1));
    }

    #[test]
    fn test_refresh_bindings() {
        let mut tc = constaint![(Any(12)) [(Any(89))] Int (Any(89))];
        tc.refresh_bindings();

        assert_eq!(tc.types[0].get_binding(), Some(0));
        assert_eq!(tc.types[1].get_binding(), Some(1));
        assert_eq!(tc.types[2].get_binding(), None);
        assert_eq!(tc.types[3].get_binding(), Some(1));
    }

    // #[test]
    // fn test_infer_bindings() {
    //     let mut tc1 = constaint![[(Any(0))] (Any(0))];
    //     let mut tc2 = constaint![[(Any(0))] (Any(1))];
    //     tc1.infer_bindings(&mut tc2);
    // }

    #[test]
    fn test_update_binding() {
        let mut tc = constaint![(Any(0)) [(Any(1))] Int (Any(1))];
        tc.update_binding(0, vtype!(Pos));
        tc.update_binding(1, vtype!(Color));
        tc.update_binding(2, vtype!(Int));  // this should do nothing

        assert_eq!(tc.types[0], vtype!(Pos));
        assert_eq!(tc.types[1], vtype!([Color]));
        assert_eq!(tc.types[2], vtype!(Int));
        assert_eq!(tc.types[3], vtype!(Color));
    }

    #[test]
    fn test_intersect() {
        // basic
        let tc1 = constaint![(Any(1)) (Any(1))];
        let tc2 = constaint![Int (Any(0))];
        let tc3 = constaint![(Any(0)) Int];
        let out = constaint![Int Int];
        assert_eq!(tc1.clone().intersect(tc2), Some(out.clone()));
        assert_eq!(tc1.clone().intersect(tc3), Some(out.clone()));
        // with 1 backtrace
        let tc1 = constaint![(Any(1)) (Any(1)) (Any(1))];
        let tc2 = constaint![(Any(1)) (Any(1)) Int];
        assert_eq!(tc1.clone().intersect(tc2.clone()).unwrap(), constaint![Int Int Int]);
        assert_eq!(tc2.clone().intersect(tc1.clone()).unwrap(), constaint![Int Int Int]);
        // complex
        let tc1 = constaint![(Any(1)) [(Any(3))] (Any(2)) (Any(2)) [Int]];
        let tc2 = constaint![(Any(1)) (Any(1)) (Any(2)) (Any(1)) (Any(2))];
        assert!(tc1.intersect(tc2).unwrap().strictly_matches(&constaint![[Int] [Int] [Int] [Int] [Int]]));
        // unsat
        let tc1 = constaint!((Any(1)) (Any(1)) Int Pos);
        let tc2 = constaint!((Any(0)) (Any(1)) (Any(1)) (Any(0)));
        assert_eq!(tc1.intersect(tc2), None);
        // unsat
        let tc1 = constaint!((Any(1)) [Any(1)]);
        let tc2 = constaint!((Any(0)) (Any(0)));
        assert_eq!(tc1.intersect(tc2), None);
        // problematic
        let tc1 = constaint!([Any(0)] (Any(0)));
        let tc2 = constaint!([Any(0)] (Any(1)));
        assert!(tc1.intersect(tc2).unwrap().strictly_matches(&constaint![[Any(0)] (Any(0))]));
    }
}
