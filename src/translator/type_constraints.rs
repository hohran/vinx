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

    /// [ Any(12), [Any(89)], Int, Any(89) ] -> [ Any(0), [Any(1)], Int, Any(1) ]
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

    fn update_binding(&mut self, binding: usize, new_type: VariableType) -> Vec<usize> {
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

    fn infer_intersect_bindings(&mut self, other: &mut Self) {
        println!("{self} + {other}");
        let mut to_intersect: Vec<usize> = (0..self.types.len()).collect();
        while !to_intersect.is_empty() {
            let i = to_intersect.pop().unwrap();
            if self.types[i].is_subset_of(&other.types[i]) {
                let common_depth = other.types[i].common_depth(&self.types[i]);
                // println!("update binding of {:?} to {:?}", other.types[i], self.types[i].unwrap_depth(common_depth).with_inverted_binding());
                let updated = other.update_binding(
                    other.types[i].get_binding()                     // binding of current type
                        .expect("error: expected to be ambiguous")   // if it is a superset, it should always be ambiguous (it will have a binding number)
                    ,self.types[i]                       // for the current type
                        .unwrap_depth(common_depth)     // this is the part to update with
                        .with_inverted_binding()        // invert binding so that it does not get mixed up with currently used bindings (if it is ambiguous)
                );
                for u in &updated {
                    if *u < i {
                        to_intersect.push(*u);
                    }
                }
                // to_intersect.append(&mut updated);
            }
            if other.types[i].is_subset_of(&self.types[i]) {
                let common_depth = self.types[i].common_depth(&other.types[i]);
                let mut updated = self.update_binding(self.types[i].get_binding().expect("error: expected to be ambiguous"), other.types[i].unwrap_depth(common_depth).with_inverted_binding());
                to_intersect.append(&mut updated);
            }
        }
    }

    /// [ Any(1), Any(1) ] + [ Any(2), Int ] -> [ Int, Int ]
    pub fn intersect(mut self, mut other: Self) -> Option<Self> {
        assert_eq!(self.types.len(), other.types.len());
        let mut types: Vec<VariableType> = vec![];
        self.infer_intersect_bindings(&mut other);
        for i in 0..self.types.len() {
            if let Some(vt) = self.types[i].intersect(&other.types[i]) {
                types.push(vt);
            } else {
                return None;
            }
        }
        let mut out = Self::from(types);
        out.refresh_bindings();
        println!("intersect: {self} + {other} => {out}");
        Some(out)
    }

    pub fn intersect_var(&mut self, var_type: &VariableType, var_id: usize) -> bool {
        assert!(var_id < self.types.len(), "variable with invalid id {var_id}");
        let var = &self.types[var_id];
        if let Some(prod) = var.intersect(var_type) {
            println!("    {var:?} + {var_type:?} -> {prod:?}");
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
        assert_eq!( tc, TypeConstraints::new(2));
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
    fn test_intersect() {
        // basic
        let tc1 = constaint![(Any(1)) (Any(1))];
        let tc2 = constaint![Int (Any(1))];
        let tc3 = constaint![(Any(1)) Int];
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
        println!("\nthis:");
        assert_eq!(tc1.clone().intersect(tc2.clone()).unwrap(), constaint![[Int] [Int] [Int] [Int] [Int]]);
        // assert_eq!(tc2.clone().intersect(tc1.clone()).unwrap(), constaint![[Int] [Int] [Int] [Int] [Int]]);
    }
}
