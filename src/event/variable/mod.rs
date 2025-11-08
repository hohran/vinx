pub mod types;
pub mod values;

use std::collections::HashMap;

use values::VariableValue;

use crate::context::Context;

use super::component::Component;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VariableLocation {
    Scope,
    Static,
    Component,
}

#[derive(Clone, Debug)]
pub struct Variable {
    name: String,
    location: VariableLocation,
    val: Option<VariableValue>,
}

impl Variable {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), location: VariableLocation::Scope, val: None }
    }

    pub fn new_static(val: VariableValue) -> Self {
        Self { name: "".to_string(), location: VariableLocation::Static, val: Some(val) }
    }

    pub fn new_component(name: &str) -> Self {
        Self { name: name.to_string(), location: VariableLocation::Component, val: None }
    }

    pub fn is_static(&self) -> bool {
        self.location == VariableLocation::Static
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_location(&self) -> VariableLocation {
        self.location
    }

    pub fn get_value(&self, _context: &Context, scope: &mut HashMap<String, VariableValue>) -> VariableValue {
        match self.location {
            VariableLocation::Static => { 
                if let Some(v) = &self.val {
                    v.clone()
                } else {
                    panic!("error: could not find value of variable {}", &self.name);
                }
            }
            VariableLocation::Scope => { 
                match scope.get(&self.name) {
                    Some(v) => {
                        v.clone()
                    }
                    None => { 
                        panic!("error: could not find value of variable {}", &self.name);
                    }
                }
            }
            VariableLocation::Component => {
                if let Some(v) = &self.val {
                    v.clone()
                } else {
                    panic!("error: could not get component value");
                }
            }
        }
    }

    pub fn set_value(&self, _context: &mut Context, scope: &mut HashMap<String, VariableValue>, new_val: VariableValue) {
        let v1_val = match self.location {
            VariableLocation::Static => { panic!("error: cannot mutate a static variable") }
            VariableLocation::Scope => {
                scope.get(&self.name)
            }
            VariableLocation::Component => {
                panic!("cannot set a component");   // FIXME maybe not necessary
            }
        }.expect("variable not found");
        if v1_val.type_check(&new_val) == false {
            panic!("values are of incompatible types");
        }
        match self.location {
            VariableLocation::Static => { panic!("cannot add to static variable") } // maybe add warning that static variables are immutable
            VariableLocation::Scope => {
                scope.insert(self.name.clone(), new_val);
            }
            VariableLocation::Component => { }
        }
    }

    pub fn to_string(&self, _context: &Context, scope: &HashMap<String, VariableValue>, _components: &mut HashMap<String,Component>) -> String {
        match self.location {
            VariableLocation::Static => {
                if let Some(v) = &self.val {
                    return v.to_string();
                } else {
                    panic!("unset static value");
                }
            }
            VariableLocation::Scope => { 
                scope.get(&self.name)
            }
            VariableLocation::Component => {
                return "component".to_string();
            }
        }.expect("unset value").to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::{event::variable::types::VariableType, vtype};

    #[test]
    fn test_variable_type_macro() {
        let t1 = vtype!(Int);
        assert_eq!(t1, VariableType::Int);
        let t2 = vtype!(Label);
        assert_eq!(t2, VariableType::Label);
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
