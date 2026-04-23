use std::fmt::Display;

use super::{ VariableValue,VariableType,Stack };

/// Representation of variables
/// Static -> literals, such as 1, "hey", red
/// Named  -> variables on the stack
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
    pub fn get_value_mut(&'a mut self, stack: &'a mut Stack) -> &'a mut VariableValue {
        match self {
            Variable::Static(v) => v,
            Variable::Named(n, t) => {
                let res = stack.get_variable_of_type_mut(n, t);
                match res {
                    Some(v) => v,
                    None => panic!("error: could not find value of variable {n} ({t})")
                }
            }
        }
    }

    /// Get a reference to the value of this variable.
    pub fn get_value(&'a self, stack: &'a Stack) -> &'a VariableValue {
        match self {
            Variable::Static(v) => v,
            Variable::Named(n, t) => {
                let res = stack.get_variable_of_type(n, t);
                match res {
                    Some(v) => v,
                    None => panic!("error: could not find value of variable {n} ({t})")
                }
            }
        }
    }

    /// Set a new value to this variable.
    /// This value has to be of a matching type.
    pub fn set_value(&mut self, stack: &mut Stack, new_val: VariableValue) {
        let val = self.get_value_mut(stack);
        if !new_val.is_assignable_to(val) {
            panic!("error: values are of incompatible types: {val} vs {new_val}");
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
            Self::Static(v) =>write!(f, "{v}"),
            Self::Named(n, _) => write!(f, "{n}"),
        }
    }
}
