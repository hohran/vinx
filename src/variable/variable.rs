use std::fmt::Display;

use super::{ VariableValue,VariableType,Stack };

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Variable {
    Static(VariableValue),
    Named(String, VariableType),
}

impl<'a> Variable {
    /// Create new named variable.
    pub fn new(name: &str, typ: VariableType) -> Self {
        Self::Named(name.to_string(), typ)
    }

    /// Create new static variable.
    pub fn new_static(val: VariableValue) -> Self {
        Self::Static(val)
    }

    /// Get variable name (empty for static variables).
    pub fn get_name(&self) -> &str {
        match self {
            Variable::Named(n, _) => &n,
            _ => "",
        }
    }

    pub fn is_on_stack(&self) -> bool {
        match self {
            Variable::Named(_, _) => true,
            _ => false,
        }
    }

    /// Get a mutable reference to the value of this variable.
    pub fn get_value_mut(&'a mut self, scope: &'a mut Stack) -> &'a mut VariableValue {
        match self {
            Variable::Static(v) => v,
            Variable::Named(n, t) => {
                let res = scope.get_variable_of_type_mut(n, t);
                match res {
                    Some(v) => v,
                    None => panic!("error: could not find value of variable {} ({})", n, t.to_string())
                }
            }
        }
    }

    /// Get a reference to the value of this variable.
    pub fn get_value(&'a self, scope: &'a Stack) -> &'a VariableValue {
        match self {
            Variable::Static(v) => v,
            Variable::Named(n, t) => {
                let res = scope.get_variable_of_type(n, t);
                match res {
                    Some(v) => v,
                    None => panic!("error: could not find value of variable {} ({})", n, t.to_string())
                }
            }
        }
    }

    /// Set a new value to this variable.
    /// This value has to be of a matching type.
    pub fn set_value(&mut self, scope: &mut Stack, new_val: VariableValue) {
        let val = self.get_value_mut(scope);
        if !new_val.is_assignable_to(val) {
            panic!("error: values are of incompatible types: {val:?} vs {new_val:?}");
        }
        *val = new_val;
    }

    pub fn get_type(&self) -> VariableType {
        match self {
            Self::Static(v) => v.get_type(),
            Self::Named(_, t) => t.clone(),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static(v) =>write!(f, "{}", v.to_string()),
            Self::Named(n, _) => write!(f, "{n}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{variable::types::VariableType, vtype};

    #[test]
    fn test_variable_type_macro() {
        let t1 = vtype!(Int);
        assert_eq!(t1, VariableType::Int);
        let t2 = vtype!(String);
        assert_eq!(t2, VariableType::String);
        let t3 = vtype!(Pos);
        assert_eq!(t3, VariableType::Pos);
        let t4 = vtype!(Color);
        assert_eq!(t4, VariableType::Color);
        let t5 = vtype!(Direction);
        assert_eq!(t5, VariableType::Direction);
        let t6 = vtype!(Any(3));
        assert_eq!(t6, VariableType::Any(3));
        let t7 = vtype!([Int]);
        assert_eq!(t7, VariableType::Vec(Box::new(VariableType::Int)));
        assert!(vtype!([Any(2)]).is_ambiguous());
        assert_eq!(vtype!([[Int]]), VariableType::Vec(Box::new(VariableType::Vec(Box::new(VariableType::Int)))));
    }
}
