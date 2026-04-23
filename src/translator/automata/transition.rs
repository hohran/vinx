use crate::{translator::{Word, type_constraints::TypeConstraints}, variable::VariableType};

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct Transition ( Word );

impl From<Word> for Transition {
    fn from(value: Word) -> Self {
        Self(value)
    }
}

pub type Applicability = usize;

impl Transition {
    /// Check if transition is applicable for word, given the current binding for `Any` types.
    #[allow(dead_code)]
    pub fn is_applicable(&self, word: &Word, binding: &TypeConstraints) -> bool {
        assert!(!word.is_ambiguous());
        if !self.0.is_ambiguous() {
            return self.0 == *word;
        }
        let Word::Type(word_t) = word else {
            return false;
        };
        let self_type = self.0.get_type().unwrap();
        if self_type.get_depth() > word_t.get_depth() {
            return false;
        }
        let word_t = word_t.unwrap_depth(self_type.get_depth());
        let b = self_type.get_binding().unwrap();
        let transition = binding.at(b);
        if transition.is_ambiguous() {
            transition.get_depth() <= word_t.get_depth()
        } else {
            &transition == word_t
        }
    }

    /// Get the applicability of transition for a word, given binding for `Any` types (None for inapplicable).
    pub fn get_applicability(&self, word: &Word, binding: &TypeConstraints) -> Option<Applicability> {
        if self.is(word) {
            return Some(Applicability::MAX);
        }
        if word.is_ambiguous() {
            return None;
        }
        let Some(transition_t) = self.0.get_type() else {
            return None;
        };
        if !transition_t.is_ambiguous() {
            return None;
        }
        // If it was a keyword, it would have matched before
        let Some(word_t) = word.get_type() else {
            return None;
        };
        let transition_depth = transition_t.get_depth();
        if transition_depth > word_t.get_depth() {
            return None;
        }
        let word_t = word_t.unwrap_depth(transition_depth);
        let bind_transition = &binding.at(transition_t.get_binding().unwrap());
        if bind_transition == word_t {
            return Some(Applicability::MAX-1);
        }
        if !bind_transition.is_ambiguous() {
            return None;
        }
        let bind_transition_depth = bind_transition.get_depth();
        if bind_transition_depth > word_t.get_depth() {
            return None;
        }
        Some(transition_depth+bind_transition_depth)
    }

    /// If this transition is Word::Type(t), get the type `t`.
    /// Otherwise panic: you should have checked beforehand.
    pub fn get_type(&self) -> &VariableType {
        self.0.get_type().unwrap()
    }

    /// Checks if the inner word is for type.
    pub fn is_type(&self) -> bool {
        self.0.is_type()
    }

    /// Checks if the word `word` specifies the transition.
    pub fn is(&self, word: &Word) -> bool {
        self.0.strictly_matches(word)
    }

    /// Get the word specifying the transition.
    pub fn get(&self) -> &Word {
        &self.0
    }

    /// Checks if the inner word is an ambiguous type.
    pub fn is_ambiguous(&self) -> bool {
        self.0.is_ambiguous()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{vtype,word};

    #[test]
    fn test_is_applicable() {
        // keywords
        let empty_bind = &TypeConstraints::new();
        assert!(Transition::from(word!("a")).is_applicable(&word!("a"), empty_bind));
        assert!(!Transition::from(word!("b")).is_applicable(&word!("a"), empty_bind));
        // types
        // // without binding
        let t_int = Transition::from(word!(Int));
        let t_pos = Transition::from(word!(Pos));
        let t_vint = Transition::from(word!([Int]));
        let t_a0 = Transition::from(word!(Any(0)));
        let t_a1 = Transition::from(word!(Any(1)));
        let t_a2 = Transition::from(word!(Any(2)));
        let t_va0 = Transition::from(word!([Any(0)]));
        assert!(t_int.is_applicable(t_int.get(), empty_bind));
        assert!(!t_int.is_applicable(t_pos.get(), empty_bind));
        assert!(!t_int.is_applicable(t_vint.get(), empty_bind));
        assert!(!t_vint.is_applicable(t_int.get(), empty_bind));
        assert!(t_vint.is_applicable(t_vint.get(), empty_bind));
        assert!(!t_pos.is_applicable(t_int.get(), empty_bind));
        assert!(t_va0.is_applicable(t_vint.get(), empty_bind));
        assert!(!t_va0.is_applicable(t_int.get(), empty_bind));
        // // with binding
        let binding = TypeConstraints::from(vec![
            vtype!(Int),     // 0 => Int
            vtype!([Int]),   // 1 => [Int]
            vtype!(Pos),     // 2 => Pos
        ]);
        assert!(t_a0.is_applicable(t_int.get(), &binding));
        assert!(!t_a1.is_applicable(t_int.get(), &binding));
        assert!(!t_a2.is_applicable(t_int.get(), &binding));
        assert!(!t_a0.is_applicable(t_vint.get(), &binding));
        assert!(t_a1.is_applicable(t_vint.get(), &binding));
        assert!(t_a2.is_applicable(t_pos.get(), &binding));
        // Sanity check - binding does not matter with non-ambiguous types
        assert!(t_int.is_applicable(t_int.get(), &binding));
        assert!(!t_int.is_applicable(t_pos.get(), &binding));
        assert!(!t_int.is_applicable(t_vint.get(), &binding));
        assert!(!t_vint.is_applicable(t_int.get(), &binding));
        assert!(t_vint.is_applicable(t_vint.get(), &binding));
        assert!(!t_pos.is_applicable(t_int.get(), &binding));
        assert!(t_va0.is_applicable(t_vint.get(), &binding));
        assert!(!t_va0.is_applicable(t_int.get(), &binding));
    }

    #[test]
    fn test_get_applicability() {
        // keywords
        let empty_bind = &TypeConstraints::new();
        let t_a = Transition::from(word!("a"));
        let t_b = Transition::from(word!("b"));
        assert!(t_a.get_applicability(t_a.get(), empty_bind).is_some());
        assert!(t_a.get_applicability(t_b.get(), empty_bind).is_none());
        assert!(t_b.get_applicability(t_b.get(), empty_bind).is_some());
        assert!(t_b.get_applicability(t_a.get(), empty_bind).is_none());
        // types
        let t_int = Transition::from(word!(Int));
        let t_vint = Transition::from(word!([Int]));
        let t_pos = Transition::from(word!(Pos));
        let t_a0 = Transition::from(word!(Any(0)));
        let t_a1 = Transition::from(word!(Any(1)));
        let t_a2 = Transition::from(word!(Any(2)));
        let t_a3 = Transition::from(word!(Any(3)));
        let t_va0 = Transition::from(word!([Any(0)]));
        let binding = TypeConstraints::from(vec![
            vtype!(Int),     // 0 => Int
            vtype!([Int]),   // 1 => [Int]
            vtype!(Any(0)),  // 2 => Any(0)
            vtype!([Any(0)]) // 3 => [Any(0)]
        ]);
        // check match on identity
        assert!(t_int.get_applicability(t_int.get(), &binding).is_some());
        assert!(t_vint.get_applicability(t_vint.get(), &binding).is_some());
        assert!(t_pos.get_applicability(t_pos.get(), &binding).is_some());
        // check no match on different
        assert!(t_int.get_applicability(t_pos.get(), &binding).is_none());
        assert!(t_int.get_applicability(t_vint.get(), &binding).is_none());
        assert!(t_vint.get_applicability(t_int.get(), &binding).is_none());
        assert!(t_vint.get_applicability(t_pos.get(), &binding).is_none());
        assert!(t_pos.get_applicability(t_int.get(), &binding).is_none());
        assert!(t_pos.get_applicability(t_vint.get(), &binding).is_none());
        // check empty binding any match
        assert!(t_a0.get_applicability(t_int.get(), empty_bind).is_some());
        assert!(t_a0.get_applicability(t_vint.get(), empty_bind).is_some());
        assert!(t_a0.get_applicability(t_pos.get(), empty_bind).is_some());
        assert!(t_a1.get_applicability(t_int.get(), empty_bind).is_some());
        assert!(t_a1.get_applicability(t_vint.get(), empty_bind).is_some());
        assert!(t_a1.get_applicability(t_pos.get(), empty_bind).is_some());
        assert!(t_va0.get_applicability(t_vint.get(), empty_bind).is_some());
        assert!(t_va0.get_applicability(t_int.get(), empty_bind).is_none());
        assert!(t_va0.get_applicability(t_pos.get(), empty_bind).is_none());
        // any with binding
        assert!(t_a0.get_applicability(t_int.get(), &binding).is_some());
        assert!(t_a0.get_applicability(t_vint.get(), &binding).is_none());
        assert!(t_a0.get_applicability(t_pos.get(), &binding).is_none());
        assert!(t_a1.get_applicability(t_int.get(), &binding).is_none());
        assert!(t_a1.get_applicability(t_vint.get(), &binding).is_some());
        assert!(t_a1.get_applicability(t_pos.get(), &binding).is_none());
        assert!(t_va0.get_applicability(t_int.get(), &binding).is_none());
        assert!(t_va0.get_applicability(t_vint.get(), &binding).is_some());
        assert!(t_va0.get_applicability(t_pos.get(), &binding).is_none());
        // ordering
        let int_int = t_int.get_applicability(t_int.get(), &binding).unwrap();
        let aint_int = t_a0.get_applicability(t_int.get(), &binding).unwrap();
        let a_int = t_a2.get_applicability(t_int.get(), &binding).unwrap();
        let aa_vint = t_a2.get_applicability(t_vint.get(), &binding).unwrap();
        let ava_vint = t_a3.get_applicability(t_vint.get(), &binding).unwrap();
        let va_vint = t_va0.get_applicability(t_vint.get(), &binding).unwrap();
        assert!(int_int > aint_int);
        assert!(int_int > a_int);
        assert!(aint_int > a_int);
        assert!(va_vint > aa_vint);
        assert!(va_vint > ava_vint);
        assert!(ava_vint > aa_vint);
    }
}
