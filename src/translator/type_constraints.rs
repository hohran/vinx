use std::fmt::Display;

use crate::variable::types::VariableType;

#[derive(Debug,PartialEq, Eq, Clone)]
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

    pub fn from(types: Vec<VariableType>) -> Self {
        Self { types }
    }

    pub fn get_types(&self) -> &Vec<VariableType> {
        &self.types
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

    // Refine all types of with given binding with new_type.
    // Returns: Vector of all changed variables
    pub fn update_binding(&mut self, binding: usize, new_type: VariableType) -> Vec<usize> {
        let mut out = vec![];
        for i in 0..self.types.len() {
            let vt = &mut self.types[i];
            let old = vt.clone();
            vt.set_binding_type(binding, &new_type);
            if !old.strictly_matches(&vt) {
                out.push(i);
            }
        }
        out
    }

    /// If self[at] (e.g., Any(0)) is a superset of other[at] (e.g., Int), replace every occurrence
    /// of self[at] in self, with other[at]
    /// so self = [Any(0) Any(0) Pos] would change to [Int Int Pos]
    /// Returns: Vector of changed types (in our examle: [0,1])
    fn infer_bindings_at(&mut self, other: &Self, at: usize) -> Vec<usize> {
        let self_at = &self.types[at];
        let other_at = &other.types[at];
        if other.types[at].is_subset_of(&self.types[at]) {
            // disallow that Any(0) is assignable with [Any(0)]
            if self_at.get_binding() == other_at.with_inverted_binding().get_binding() && self_at.get_depth() != other_at.get_depth() {
                self.types[at] = VariableType::None;
                return vec![];
            }
            let depth = self.types[at].get_depth();     // since self in other: other has always lower depth
            let mut updated = self.update_binding(
                self.types[at].get_binding()                   // binding of current type
                .expect("error: expected to be ambiguous"),  // if it is a superset, it should always be ambiguous (it will have a binding number)
                other.types[at]                  // for the current type
                .unwrap_depth(depth)            // this is the part to update with
                .with_inverted_binding()        // invert binding so that it does not get mixed up with currently used bindings (if it is ambiguous)
            );
            // remove currently processed index
            if let Some(this) = updated.iter().position(|x| *x == at) {
                updated.remove(this);
            }
            updated
        } else {
            vec![]
        }
    }

    /// Iterativelly refine ambiguous datatypes of both type constraints.
    /// Example:
    /// self  = \[Any(0)  Int   Any(0)]
    /// other = \[Any(0) Any(1) Any(1)]
    /// after, both are \[Int Int Int]
    /// This function does not throw errors for nonintersectable types, 
    /// so self = \[Int] and other = \[Pos] would do nothing.
    fn infer_bindings(&mut self, other: &mut Self) {
        let mut to_intersect: Vec<usize> = (0..self.types.len()).collect();
        while !to_intersect.is_empty() {
            let i = to_intersect.pop().unwrap();
            let mut updated = self.infer_bindings_at(other, i);
            to_intersect.append(&mut updated);
            updated = other.infer_bindings_at(self, i);
            to_intersect.append(&mut updated);
        }
    }

    /// [ Any(1), Any(1) ] + [ Any(2), Int ] -> [ Int, Int ]
    /// [Int Pos] + [Any(0) Any(0)] -> None
    pub fn intersect(mut self, mut other: Self) -> Option<Self> {
        assert_eq!(self.types.len(), other.types.len());
        let mut types: Vec<VariableType> = vec![];
        self.infer_bindings(&mut other);
        for i in 0..self.types.len() {
            if let Some(vt) = self.types[i].intersect(&other.types[i]) {
                types.push(vt);
            } else {
                return None;
            }
        }
        let mut out = Self::from(types);
        out.refresh_bindings();
        Some(out)
    }

    /// Intersects variable with var_id.
    /// Returns if it is intersectable with var_type
    pub fn intersect_var(&mut self, var_type: &VariableType, var_id: usize) -> bool {
        assert!(var_id < self.types.len(), "variable with invalid id {var_id}");
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
        assert_eq!(tc1.clone().intersect(tc2.clone()).unwrap(), constaint![[Int] [Int] [Int] [Int] [Int]]);
        // unsat
        let tc1 = constaint!((Any(1)) (Any(1)) Int Pos);
        let tc2 = constaint!((Any(0)) (Any(1)) (Any(1)) (Any(0)));
        assert_eq!(tc1.intersect(tc2), None);
        // unsat
        let tc1 = constaint!((Any(1)) [Any(1)]);
        let tc2 = constaint!((Any(0)) (Any(0)));
        assert_eq!(tc1.intersect(tc2), None);
    }
}
