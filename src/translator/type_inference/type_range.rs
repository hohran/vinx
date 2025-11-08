use crate::event::variable::types::VariableType;


#[derive(Clone,Debug,PartialEq, Eq)]
pub struct TypeRange {
    possible_types: Vec<VariableType>,
}

impl TypeRange {
    pub fn new(var_id: usize) -> Self {
        Self { possible_types: vec![VariableType::Any(var_id)] }
    }

    pub fn empty() -> Self {
        Self { possible_types: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.possible_types.is_empty()
    }

    pub fn contains_ambiguous_vars(&self) -> bool {
        for t in &self.possible_types {
            if t.is_ambiguous() {
                return true;
            }
        }
        false
    }

    pub fn get(&self) -> &Vec<VariableType> {
        &self.possible_types
    }

    pub fn suck_out(self) -> Vec<VariableType> {
        self.possible_types
    }

    // unification is meant for type ranges bound to the same variable
    pub fn unify(&mut self, tr2: Self) {
        // we do not unify Pos+Any into Any, because we want to split these cases
        for t in tr2.suck_out() {
            self.unify_one(t);
        }
    }

    pub fn unify_ref(&mut self, tr2: &Self) {
        for t in &tr2.possible_types {
            self.unify_one_ref(t);
        }
    }

    pub fn unify_one_ref(&mut self, t: &VariableType) {
        if !self.possible_types.contains(t) {
            self.possible_types.push(t.clone());
        }
    }

    pub fn unify_vec(&mut self, v: Vec<VariableType>) {
        for t in v {
            self.unify_one(t);
        }
    }

    pub fn unify_one(&mut self, t: VariableType) {
        if !self.possible_types.contains(&t) {
            self.possible_types.push(t);
        }
    }

    /// Intersect a new type
    /// Return whether the intersection is non-empty
    /// * We expect that there is only one possible type at this point
    /// * This is because the decision automaton is tree-like
    pub fn intersect_one(&mut self, v: &VariableType) -> bool {
        assert!(self.possible_types.len() == 1, "error: expected to have only one possible type");
        match (&self.possible_types[0],v) {
            (VariableType::Any(_),_) => {
                self.possible_types[0] = v.clone();
                true
            }
            (_,VariableType::Any(_)) => true,
            (a,b) => {
                if let Some(x) = get_type_intersection(a, b) {
                    self.possible_types[0] = x;
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Checks if an unambiguous type is contained
    /// Examples: 
    /// Int in { Int|Pos } -> true
    /// Int in { Pos } -> false
    /// Int in { Any } -> true
    fn contains_unambiguous(&self, v: &VariableType) -> bool {
        assert!(!v.is_ambiguous(), "error: {:?} is ambiguous", v);
        for t in &self.possible_types {
            if get_type_intersection(t, v).is_some() {
                return true;
            }
        }
        false
    }

    fn contains_ambiguous(&self, v: &VariableType) -> bool {
        let v_clone = Some(v.clone());
        for t in &self.possible_types {
            let i = get_type_intersection(t, v);
            if i == v_clone {
                return true;
            }
        }
        false
    }

    fn get_ambiguous_intersect(&self, v: &VariableType) -> Self {
        let mut intersect = Self { possible_types: vec![] };
        for t in &self.possible_types {
            if let Some(x) = get_type_intersection(v, t) {
                intersect.unify_one(x);
            }
        }
        intersect
    }

    /// Calculate intersection of type ranges
    /// This is used for bound variables
    /// Example: 
    /// intersect([ { Int|Pos },{ Pos|Color },{ Any(4) } ]) = { Pos }
    /// intersect([ { Any(1) },{ Any(2) },{ Any(4) } ]) = { Any(1) }
    pub fn intersect(trs: Vec<&Self>) -> Self {
        match trs.len() {
            0 => return Self::new(1),
            1 => return trs[0].clone(),
            _ => {}
        }
        let mut intersect = Self { possible_types: vec![] }; 
        'type_iterator:
        for vt in &trs[0].possible_types {
            if !vt.is_ambiguous() {
                for tr in trs.iter().skip(1) {
                    if !tr.contains_unambiguous(vt) {
                        continue 'type_iterator;
                    }
                }
                intersect.unify_one(vt.clone());
                continue;
            }
            // ambiguous: get all possible types
            for itr1 in 1..trs.len() {
                let possible_types = trs[itr1].get_ambiguous_intersect(vt);
                'intersected_type_iterator:
                for intersected_t in possible_types.possible_types {
                    if intersect.contains_ambiguous(&intersected_t) {
                        continue;
                    }
                    for itr2 in 2..trs.len() {
                        if itr1 == itr2 {
                            continue;
                        }
                        if !trs[itr2].contains_ambiguous(&intersected_t) {
                            continue 'intersected_type_iterator;
                        }
                    }
                    intersect.unify_one(intersected_t);
                }
            }
        }
        intersect
    }
}

pub fn get_type_intersection(a: &VariableType, b: &VariableType) -> Option<VariableType> {
    if a == b {
        return Some(a.clone());
    }
    match (a,b) {
        (VariableType::Vec(x), VariableType::Vec(y)) => {
            if let Some(t) = get_type_intersection(&x, &y) {
                return Some(VariableType::Vec(Box::new(t)));
            }
            None
        },
        (VariableType::Any(_), t) | (t, VariableType::Any(_)) => {
            Some(t.clone())
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn x() {
        TypeRange::new(0);
        TypeRange::empty();
    }

    #[test]
    fn test_type_intersection() {
        let vt1 = VariableType::Int;
        let vt2 = VariableType::Int;
        let vt3 = VariableType::Pos;
        let vt4 = VariableType::Any(1);
        let vt5 = VariableType::Vec(Box::new(VariableType::Int));
        let vt6 = VariableType::Vec(Box::new(VariableType::Any(1)));
        let vt7 = VariableType::Vec(Box::new(VariableType::Pos));
        assert_eq!(get_type_intersection(&vt1, &vt2), Some(VariableType::Int));
        assert_eq!(get_type_intersection(&vt1, &vt3), None);
        assert_eq!(get_type_intersection(&vt1, &vt4), Some(VariableType::Int));
        assert_eq!(get_type_intersection(&vt4, &vt3), Some(VariableType::Pos));
        assert_eq!(get_type_intersection(&vt5, &vt6), Some(VariableType::Vec(Box::new(VariableType::Int))));
        assert_eq!(get_type_intersection(&vt5, &vt7), None);
        assert_eq!(get_type_intersection(&vt6, &vt7), Some(VariableType::Vec(Box::new(VariableType::Pos))));
        assert_eq!(get_type_intersection(&vt4, &vt6), Some(VariableType::Vec(Box::new(VariableType::Any(1)))));
        assert_eq!(get_type_intersection(&vt6, &vt4), Some(VariableType::Vec(Box::new(VariableType::Any(1)))));
    }

    #[test]
    fn test_type_range_intersect() {
        let mut tr = TypeRange::new(0);
        assert_eq!(tr.get(), &vec![VariableType::Any(0)]);
        assert!(tr.intersect_one(&VariableType::Pos));
        assert_eq!(tr.get(), &vec![VariableType::Pos]);
        assert!(!tr.intersect_one(&VariableType::Color));
    }

    #[test]
    fn test_contains_ambiguous() {
        let mut tr = TypeRange { possible_types: vec![] };
        tr.unify_vec(vec![VariableType::Int,VariableType::Pos,VariableType::Vec(Box::new(VariableType::Any(1)))]);
        assert!(tr.contains_unambiguous(&VariableType::Int));
        assert!(tr.contains_unambiguous(&VariableType::Pos));
        assert!(!tr.contains_unambiguous(&VariableType::Color));
        assert!(tr.contains_ambiguous(&VariableType::Vec(Box::new(VariableType::Any(2)))));
        assert!(!tr.contains_ambiguous(&VariableType::Any(1)));
    }

    #[test]
    fn test_intersect() {
        let tr1 = TypeRange { possible_types: vec![VariableType::Int,VariableType::Pos,VariableType::Any(1)] };
        let tr2 = TypeRange { possible_types: vec![VariableType::Int,VariableType::Any(2)] };
        let tr3 = TypeRange { possible_types: vec![VariableType::Int,VariableType::Pos,VariableType::Color] };
        let tr4 = TypeRange { possible_types: vec![VariableType::Vec(Box::new(VariableType::Any(1)))] };
        assert_eq!(TypeRange::intersect(vec![&tr1,&tr2,&tr3]).possible_types,vec![VariableType::Int,VariableType::Pos,VariableType::Color]);
        assert_eq!(TypeRange::intersect(vec![&tr1,&tr2]).possible_types,vec![VariableType::Int,VariableType::Pos,VariableType::Any(1)]);
        assert_eq!(TypeRange::intersect(vec![&tr2,&tr3]).possible_types,vec![VariableType::Int,VariableType::Pos,VariableType::Color]);
        assert_eq!(TypeRange::intersect(vec![&tr1,&tr2,&tr4]).possible_types,vec![VariableType::Vec(Box::new(VariableType::Any(1)))]);  // FIXME : this should be empty (prolly)
    }
}
