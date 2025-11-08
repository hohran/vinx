// TODO TODO TODO: figure out type bounds for [Any(1)] and Any(1)
use std::collections::HashMap;


use crate::event::variable::types::VariableType;

use super::type_range::TypeRange;

#[derive(Clone,Debug,PartialEq, Eq)]
pub struct TypeBounds {
    /// Each type range corresponds to a variable.
    type_ranges: Vec<TypeRange>,
    /// Each type identity group corresponds to a set of variables that must have the same type.
    /// Each group is represented as a vector of variable indices.
    type_identity_groups: Vec<Vec<usize>>,
}

impl TypeBounds {
    pub fn new(var_count: usize) -> Self {
        let mut type_ranges = vec![];
        for i in 0..var_count {
            type_ranges.push(TypeRange::new(i));
        }
        Self { type_ranges, type_identity_groups: vec![] }
    }

    pub fn var_count(&self) -> usize {
        self.type_ranges.len()
    }

    pub fn is_ambiguous(&self) -> bool {
        for tr in &self.type_ranges {
            if tr.get().len() > 1 {
                return true;
            }
        }
        false
    }

    pub fn contains_ambiguous_vars(&self) -> bool {
        for tr in &self.type_ranges {
            if tr.contains_ambiguous_vars() {
                return true;
            }
        }
        false
    }

    pub fn of_var(&self, var_id: usize) -> &TypeRange {
        &self.type_ranges[var_id]
    }

    pub fn ranges(&self) -> &Vec<TypeRange> {
        &self.type_ranges
    }

    pub fn empty(var_count: usize) -> Self {
        Self { type_ranges: vec![TypeRange::empty();var_count], type_identity_groups: vec![] }
    }

    pub fn empty_out(self) -> Self {
        Self { type_ranges: vec![TypeRange::empty();self.type_ranges.len()], type_identity_groups: vec![] }
    }

    /// Returns whether any variable has empty type range
    pub fn is_unsat(&self) -> bool {
        for tr in &self.type_ranges {
            if tr.is_empty() {
                return true;
            }
        }
        false
    }

    /// Checks if identity group is not of kind [1,1]
    /// Note: with [1,2,1], the redundant 1 is not cleared, but this case should not appear irl
    fn identity_group_is_nontrivial(identity_group: &Vec<usize>) -> bool {
        match identity_group.len() {
            0 | 1 => false,
            len => {
                let v = identity_group[0];
                for i in 1..len {
                    if v != identity_group[i] {
                        return true;
                    }
                }
                false
            }
        }
    }

    // TODO change to (usize, usize) pairs
    pub fn add_identity_group(&mut self, identity_group: Vec<usize>) {
        if self.type_identity_groups.len() == 0 {
            self.type_identity_groups.push(identity_group);
            return;
        }
        if self.type_identity_groups.len() == 1 {
            // check if there is a common var
            if TypeBounds::identity_groups_common_var(&identity_group, &self.type_identity_groups[0]) { 
                TypeBounds::merge_identity_groups(&mut self.type_identity_groups[0], identity_group);
            } else {
                self.type_identity_groups.push(identity_group);
            }
            return;
        }
        // transitive closure
        let mut merged_group_ids: Vec<usize> = vec![];
        for (i,g) in self.type_identity_groups.iter().enumerate() {
            if TypeBounds::identity_groups_common_var(&identity_group, g) {
                merged_group_ids.push(i);
            }
        }
        if merged_group_ids.len() == 0 {
            self.type_identity_groups.push(identity_group);
            return;
        }
        if merged_group_ids.len() == 1 {
            let group_id = merged_group_ids[0];
            TypeBounds::merge_identity_groups(&mut self.type_identity_groups[group_id], identity_group);
            return;
        }
        let mut merged_identity_group = identity_group;
        for group_id in merged_group_ids.iter().rev() {  // merge and remove groups
                                                         // we use reverse, because the group ids
                                                         // are ordered increasingly.
                                                         // this would result in unwanted groups
                                                         // being removed instead + possible segfault
            for v in &self.type_identity_groups[*group_id] {
                if !merged_identity_group.contains(v) {
                    merged_identity_group.push(*v);
                }
            }
            self.type_identity_groups.remove(*group_id);
        }
        self.type_identity_groups.push(merged_identity_group);
    }


    fn identity_groups_common_var(ig1: &Vec<usize>, ig2: &Vec<usize>) -> bool {
        for v in ig1 {
            if ig2.contains(v) {
                return true;
            }
        }
        false
    }

    fn merge_identity_groups(ig1: &mut Vec<usize>, ig2: Vec<usize>) {
        for v in ig2 {
            if ig1.contains(&v) {
                continue;
            }
            ig1.push(v);
        }
    }

    /// Intersect the type range of a variable with a new type.
    /// Return whether the intersection is non-empty.
    pub fn intersect_variable_type(&mut self, var_id: usize, var_type: &VariableType) -> bool {
        self.type_ranges[var_id].intersect_one(var_type)
    }

    pub fn intersect_bounds_noninline(&self, tb2: &TypeBounds, usages: &Vec<bool>) -> TypeBounds {
        assert!(self.type_ranges.len() == tb2.type_ranges.len(), "error: mismatch in variable count: {} vs {}", self.type_ranges.len(), tb2.type_ranges.len());
        let mut new_type_ranges = vec![];
        let mut i = 0;
        for tr2 in &tb2.type_ranges {
            let new_tr = TypeRange::intersect(vec![&self.type_ranges[i], tr2]);
            new_type_ranges.push(new_tr);
            i += 1;
        }
        let mut t = TypeBounds { type_ranges: new_type_ranges, type_identity_groups: vec![] };
        for id_group in self.infer_type_identites_with_usages(usages) {
            println!("adding identity group from tb1: {:?}", id_group);
            t.add_identity_group(id_group);
        }
        for id_group in tb2.infer_type_identites_with_usages(usages) {
            println!("adding identity group from tb2: {:?}", id_group);
            t.add_identity_group(id_group);
        }
        t.apply_type_identities();
        t
    }

    fn infer_type_identites(&self) -> Vec<Vec<usize>> {
        for tr in &self.type_ranges { assert!(tr.get().len() == 1, "error: cannot infer type identities from non-singleton type ranges"); }
        println!("infering type identities for type bounds: {:?}", self);
        let mut identity_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..self.type_ranges.len() {
            let vt = &self.type_ranges[i].get()[0];
            match vt {
                VariableType::Any(id) => {
                    if identity_map.contains_key(id) {
                        let v = identity_map.get_mut(id).unwrap();
                        v.push(i);
                    } else {
                        identity_map.insert(*id, vec![i]);
                    }
                }
                _ => {}
            }
        }
        println!("inferred identity map: {:?}", identity_map);
        let mut out = vec![];
        for identity_group in identity_map.values() {
            if identity_group.len() > 1 {
                out.push(identity_group.clone());
            }
        }
        out
    }

    pub fn infer_type_identites_with_usages(&self, usages: &Vec<bool>) -> Vec<Vec<usize>> {
        assert!(self.type_ranges.len() == usages.len());
        let mut identity_groups = vec![];
        let mut identity_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..usages.len() {
            if !usages[i] { continue; }
            let vt = &self.type_ranges[i].get()[0];
            if let VariableType::Any(id) = vt {
                if let Some(v) = identity_map.get_mut(id) {
                    v.push(i);
                } else {
                    identity_map.insert(*id, vec![i]);
                }
            }
        }
        // insert type identities
        for id_group in identity_map.into_values() {
            if id_group.len() <= 1 { continue; }
            identity_groups.push(id_group);
        }
        identity_groups
    }


    pub fn unify(&mut self, tb2: &TypeBounds) {
        assert!(self.type_ranges.len() == tb2.type_ranges.len(), "error: mismatch in variable count: {} vs {}", self.type_ranges.len(), tb2.type_ranges.len());
        let mut i = 0;
        for tr2 in &tb2.type_ranges {
            self.type_ranges[i].unify_ref(tr2);
            i += 1;
        }
    }

    pub fn apply_type_identities(&mut self) {
        for iti in 0..self.type_identity_groups.len() {
        println!("here");
            let mut identity_group = vec![];
            for v in &self.type_identity_groups[iti] {
                identity_group.push(&self.type_ranges[*v]);
            }
            let intersection = TypeRange::intersect(identity_group);
            for v in &self.type_identity_groups[iti] {
                self.type_ranges[*v] = intersection.clone();
            }
        }
        self.type_identity_groups.clear();
    }

    pub fn get_interpretations(&self) -> Vec<Vec<VariableType>> {
        let mut interpretations: Vec<Vec<VariableType>> = vec![];
        let mut current_interpretation: Vec<VariableType> = vec![];
        self.get_interpretations_recursive(0, &mut current_interpretation, &mut interpretations);
        interpretations
    }

    fn get_interpretations_recursive(&self, var_index: usize, current_interpretation: &mut Vec<VariableType>, interpretations: &mut Vec<Vec<VariableType>>) {
        if var_index == self.type_ranges.len() {
            interpretations.push(current_interpretation.clone());
            return;
        }
        let tr = &self.type_ranges[var_index];
        for vt in tr.get() {
            current_interpretation.push(vt.clone());
            self.get_interpretations_recursive(var_index + 1, current_interpretation, interpretations);
            current_interpretation.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vtype;
    // use crate::event::variable::VariableType;
    // use super::super::type_range::TypeRange;

    #[test]
    fn test_type_bounds_new() {
        let tb = TypeBounds::new(5);
        assert_eq!(tb.type_ranges.len(), 5);
        assert_eq!(tb.type_identity_groups.len(), 0);
    }

    #[test]
    fn test_type_bounds_add_identity_group() {
        let mut tb = TypeBounds::new(5);
        tb.add_identity_group(vec![0,1]);
        assert_eq!(tb.type_identity_groups.len(), 1);
        assert_eq!(tb.type_identity_groups[0], vec![0,1]);
        tb.add_identity_group(vec![2,3]);
        assert_eq!(tb.type_identity_groups.len(), 2);
        assert_eq!(tb.type_identity_groups[1], vec![2,3]);
    }

    #[test]
    fn test_type_bounds_add_identity_group_merge1() {
        let mut tb = TypeBounds::new(5);
        tb.add_identity_group(vec![0,1]);
        assert_eq!(tb.type_identity_groups.len(), 1);
        assert_eq!(tb.type_identity_groups[0], vec![0,1]);
        tb.add_identity_group(vec![1,2]);
        assert_eq!(tb.type_identity_groups.len(), 1);
        assert_eq!(tb.type_identity_groups[0], vec![0,1,2]);
    }

    #[test]
    fn test_type_bounds_add_identity_group_merge2() {
        let mut tb = TypeBounds::new(5);
        tb.add_identity_group(vec![0,1]);
        assert_eq!(tb.type_identity_groups.len(), 1);
        assert_eq!(tb.type_identity_groups[0], vec![0,1]);
        tb.add_identity_group(vec![2,3]);
        assert_eq!(tb.type_identity_groups.len(), 2);
        assert_eq!(tb.type_identity_groups[1], vec![2,3]);
        tb.add_identity_group(vec![1,3]);
        assert_eq!(tb.type_identity_groups.len(), 1);
        assert_eq!(tb.type_identity_groups[0], vec![1,3,2,0]);
    }

    #[test]
    fn test_type_bounds_simple() {
        let mut tb = TypeBounds::new(2);
        tb.add_identity_group(vec![0,1]);
        tb.intersect_variable_type(0, &vtype!(Int));
        tb.intersect_variable_type(1, &vtype!(Pos));
        tb.apply_type_identities();
        assert!(tb.is_unsat());
    }

    #[test]
    fn test_type_bounds() {
        let mut tb = TypeBounds::new(2);
        tb.add_identity_group(vec![0,1]);
        tb.intersect_variable_type(0, &vtype!(Int));
        tb.intersect_variable_type(1, &vtype!(Any(0)));
        tb.apply_type_identities();
        assert_eq!(tb.type_ranges[0].get(), &vec![vtype!(Int)]);
        assert_eq!(tb.type_ranges[1].get(), &vec![vtype!(Int)]);
    }

    #[test]
    fn test_intersect_bounds_noninline() {
        let usages = vec![true,true];
        // // [[Pos],[Int]] + [[Any(0)],[Int]] = [[Pos],[Int]]
        let mut tb1 = TypeBounds::new(2);
        tb1.intersect_variable_type(0, &vtype!(Pos));
        tb1.intersect_variable_type(1, &vtype!(Int));
        let mut tb2 = TypeBounds::new(2);
        tb2.intersect_variable_type(0, &vtype!(Any(0)));
        tb2.intersect_variable_type(1, &vtype!(Int));
        let tb3 = tb1.intersect_bounds_noninline(&tb2,&usages);
        assert_eq!(tb3.type_ranges[0].get(), &vec![vtype!(Pos)]);
        assert_eq!(tb3.type_ranges[1].get(), &vec![vtype!(Int)]);
        // // [[Pos],[Int]] + [[Any(0)],[Pos]] = unsat
        let mut tb4 = TypeBounds::new(2);
        tb4.intersect_variable_type(0, &vtype!(Pos));
        tb4.intersect_variable_type(1, &vtype!(Int));
        let mut tb5 = TypeBounds::new(2);
        tb5.intersect_variable_type(0, &vtype!(Any(0)));
        tb5.intersect_variable_type(1, &vtype!(Pos));
        let tb6 = tb4.intersect_bounds_noninline(&tb5,&usages);
        assert!(tb6.is_unsat());
        // // [[Any(1)],[Any(1)]] + [[Any(0)],[Int]] = [[Int],[Int]]
        let mut tb7 = TypeBounds::new(2);
        tb7.intersect_variable_type(0, &vtype!(Any(1)));
        tb7.intersect_variable_type(1, &vtype!(Any(1)));
        let mut tb8 = TypeBounds::new(2);
        tb8.intersect_variable_type(0, &vtype!(Any(0)));
        tb8.intersect_variable_type(1, &vtype!(Int));
        let tb9 = tb7.intersect_bounds_noninline(&tb8,&usages);
        assert_eq!(tb9.type_ranges[0].get(), &vec![vtype!(Int)]);
        assert_eq!(tb9.type_ranges[1].get(), &vec![vtype!(Int)]);
        // // [[Any(1)],[Any(1)]] + [[Any(0)],[Pos]] = [[Pos],[Pos]]
        let mut tb10 = TypeBounds::new(2);
        tb10.intersect_variable_type(0, &vtype!(Any(1)));
        tb10.intersect_variable_type(1, &vtype!(Any(1)));
        let mut tb11 = TypeBounds::new(2);
        tb11.intersect_variable_type(0, &vtype!(Any(0)));
        tb11.intersect_variable_type(1, &vtype!(Pos));
        let tb12 = tb10.intersect_bounds_noninline(&tb11,&usages);
        assert_eq!(tb12.type_ranges[0].get(), &vec![vtype!(Pos)]);
        assert_eq!(tb12.type_ranges[1].get(), &vec![vtype!(Pos)]);
        // // [[Any(1)],[Any(1)]] + [[Int],[Pos]] = unsat
        let mut tb13 = TypeBounds::new(2);
        tb13.intersect_variable_type(0, &vtype!(Any(1)));
        tb13.intersect_variable_type(1, &vtype!(Any(1)));
        let mut tb14 = TypeBounds::new(2);
        tb14.intersect_variable_type(0, &vtype!(Int));
        tb14.intersect_variable_type(1, &vtype!(Pos));
        let tb15 = tb13.intersect_bounds_noninline(&tb14,&usages);
        assert!(tb15.is_unsat());
        // // [[Int],[Pos]] + [[Any(1)],[Any(1)]] = unsat
        let tb16 = tb14.intersect_bounds_noninline(&tb13,&usages);
        assert!(tb16.is_unsat());
    }

    #[test]
    fn test_infer_type_identities() {
        let mut tb = TypeBounds::new(4);
        tb.intersect_variable_type(0, &vtype!(Any(1)));
        tb.intersect_variable_type(1, &vtype!(Any(1)));
        tb.intersect_variable_type(2, &vtype!(Int));
        tb.intersect_variable_type(3, &vtype!(Any(2)));
        let id_groups = tb.infer_type_identites();
        assert_eq!(id_groups.len(), 1);
        assert_eq!(id_groups[0], vec![0,1]);
    }

    #[test]
    fn test_get_interpretations() {
        let mut tb = TypeBounds::new(3);
        tb.intersect_variable_type(0, &vtype!(Int));
        tb.intersect_variable_type(1, &vtype!(Pos));
        tb.intersect_variable_type(2, &vtype!(Any(0)));
        let interpretations = tb.get_interpretations();
        assert_eq!(interpretations.len(), 1);
        let mut tb2 = TypeBounds::new(3);
        tb2.intersect_variable_type(0, &vtype!(Pos));
        tb2.intersect_variable_type(1, &vtype!(Int));
        tb2.intersect_variable_type(2, &vtype!(Int));
        let interpretations2 = tb2.get_interpretations();
        assert_eq!(interpretations2.len(), 1);
        tb.unify(&tb2);
        let interpretations3 = tb.get_interpretations();
        assert_eq!(interpretations3.len(), 8);
    }
}
