pub mod types;
pub mod values;
pub mod stack;

use stack::Stack;
use types::VariableType;
use values::VariableValue;

use crate::context::Context;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VariableLocation {
    Scope,
    Static,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    name: String,
    location: VariableLocation,
    val: Option<VariableValue>,
    typ: VariableType,
}

impl<'a> Variable {
    pub fn new(name: &str, typ: VariableType) -> Self {
        Self { name: name.to_string(), location: VariableLocation::Scope, val: None, typ }
    }

    pub fn new_static(val: VariableValue) -> Self {
        Self { name: "".to_string(), location: VariableLocation::Static, val: Some(val.clone()), typ: val.get_type() }
    }

    // pub fn new_component(name: &str, component_type: usize) -> Self {
    //     Self { name: name.to_string(), location: VariableLocation::Component, val: None, typ: VariableType::Component(component_type) }
    // }

    pub fn is_static(&self) -> bool {
        self.location == VariableLocation::Static
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_location(&self) -> VariableLocation {
        self.location
    }

    pub fn get_value(&self, scope: &Stack) -> VariableValue {
        // println!("get {} ({})", self.name, self.typ.to_string());
        match self.location {
            VariableLocation::Static => { 
                if let Some(v) = &self.val {
                    v.clone()
                } else {
                    panic!("error: could not find value of variable {}", &self.name);
                }
            }
            VariableLocation::Scope => { 
                match scope.get_variable_of_type(&self.name, self.get_type()) {
                    Some(v) => {
                        v.clone()
                    }
                    None => panic!("error: could not find value of variable {}", &self.name)
                    // None => { 
                    //     if self.name == "$width" {
                    //         return VariableValue::Int(context.get_width() as i32);
                    //     }
                    //     if self.name == "$height" {
                    //         return VariableValue::Int(context.get_height() as i32);
                    //     }
                    //     panic!("error: could not find value of variable {}", &self.name);
                    // }
                }
            }
        }
    }

    pub fn get_value_of_type(&self, scope: &Stack, var_type: &VariableType) -> VariableValue {
        match self.location {
            VariableLocation::Static => { 
                if let Some(v) = &self.val {
                    assert!(v.get_type() == *var_type);
                    v.clone()
                } else {
                    panic!("error: could not find value of variable {}", &self.name);
                }
            }
            VariableLocation::Scope => { 
                match scope.get_variable_of_type(&self.name, var_type) {
                    Some(v) => {
                        v.clone()
                    }
                    None => panic!("error: could not find value of variable {}", &self.name)
                    // None => { 
                    //     if self.name == "$width" {
                    //         return VariableValue::Int(context.get_width() as i32);
                    //     }
                    //     if self.name == "$height" {
                    //         return VariableValue::Int(context.get_height() as i32);
                    //     }
                    //     panic!("error: could not find value of variable {}", &self.name);
                    // }
                }
            }
        }
    }

    pub fn get_value_ref(&'a self, scope: &'a Stack) -> &'a VariableValue {
        match self.location {
            VariableLocation::Static => { 
                if let Some(v) = &self.val {
                    v
                } else {
                    panic!("error: could not find value of variable {}", &self.name);
                }
            }
            VariableLocation::Scope => { 
                match scope.get_variable(&self.name) {
                    Some(v) => {
                        v
                    }
                    None => { 
                        // if self.name == "$width" {   TODO add
                        //     return VariableValue::Int(context.get_width() as i32);
                        // }
                        // if self.name == "$height" {
                        //     return VariableValue::Int(context.get_height() as i32);
                        // }
                        panic!("error: could not find value of variable {}", &self.name);
                    }
                }
            }
        }
    }

    pub fn set_value(&mut self, scope: &mut Stack, new_val: VariableValue) {
        let val = match self.location {
            VariableLocation::Static => { self.val.as_mut() }
            VariableLocation::Scope => {
                scope.get_variable_of_type_mut(&self.name, self.get_type())
            }
        }.expect("variable not found");
        // if val.get_type() != new_val.get_type() {
        //     panic!("values are of incompatible types: {val:?} vs {new_val:?}");
        // }
        if val.type_check(&new_val) == false {
            panic!("values are of incompatible types: {val:?} vs {new_val:?}");
        }
        *val = new_val;
        // match self.location {
        //     VariableLocation::Static => { panic!("cannot add to static variable") } // maybe add warning that static variables are immutable
        //     VariableLocation::Scope => {
        //         scope.insert(self.name.clone(), new_val);
        //     }
        //     VariableLocation::Component => { }
        // }
    }

    pub fn get_type(&self) -> &VariableType {
        &self.typ
    }
}

impl ToString for Variable {
    fn to_string(&self) -> String {
        match self.location {
            VariableLocation::Scope => format!("{}",self.name),
            VariableLocation::Static => format!("{}",self.val.as_ref().expect("error: no value for static variable").to_string()),
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
