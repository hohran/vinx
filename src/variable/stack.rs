use std::collections::HashMap;

use crate::variable::{VariableType, VariableValue};

pub type Scope = HashMap<String,VariableValue>;

/// Stack of variable scopes.
#[derive(Debug,Clone)]
pub struct Stack {
    pub scopes: Vec<Scope>
}

impl Stack {
    /// Create a new stack with one empty scope
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()] }
    }

    /// Adds a new empty scope to the stack
    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Print statement, which is indented based on the number of scopes
    pub fn pretty_println(&self, string: String) {
        let tab = 2;
        print!("{:<1$}", "", self.scopes.len()*tab);
        println!("{string}");
    }

    /// Adds a new scope to the stack
    pub fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    /// Get the top scope of the stack
    pub fn top(&self) -> &Scope{
        let top_index = self.scopes.len()-1;
        &self.scopes[top_index]
    }

    fn top_mut(&mut self) -> &mut Scope {
        let top_index = self.scopes.len()-1;
        &mut self.scopes[top_index]
    }

    /// Pops the top scope from the stack and returns it
    pub fn pop(&mut self) -> Scope {
        assert!(!self.scopes.is_empty());
        self.scopes.pop().unwrap()
    }

    /// Adds a variable to the top scope
    /// This variable needs to be unique in the current scope
    pub fn add_variable(&mut self, name: String, value: VariableValue) {
        let top = self.top_mut();
        assert!(top.insert(name.clone(), value).is_none(), "error: cannot add variable {name} because it already exists");
    }

    /// Updates a variable on the stack with a new value
    pub fn update_variable(&mut self, name: &str, new_value: VariableValue) {
        for scope_idx in (0..self.scopes.len()).rev() {
            let val = self.scopes[scope_idx].get_mut(name);
            if let Some(v) = val {
                // assert!(new_value.is_assignable_to(v), "error: {} is not assignable to {name} ({})", new_value.to_string(), v.get_type());
                *v = new_value;
                return;
            }
        }
        panic!("error: variable \"{name}\" not found")
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableValue> {
        for scope in self.scopes.iter().rev() {
            let val = scope.get(name);
            if val.is_some() {
                return val;
            }
        }
        None
    }

    pub fn get_variable_of_type(&self, name: &str, var_type: &VariableType) -> Option<&VariableValue> {
        for scope in self.scopes.iter().rev() {
            let val = scope.get(name);
            if let Some(v) = val {
                if v.get_type().is_assignable_to(var_type) {
                    return val;
                }
            }
        }
        None
    }

    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut VariableValue> {
        for scope in self.scopes.iter_mut().rev() {
            let val = scope.get_mut(name);
            if val.is_some() {
                return val;
            }
        }
        None
    }

    pub fn get_variable_of_type_mut(&mut self, name: &str, var_type: &VariableType) -> Option<&mut VariableValue> {
        for scope in self.scopes.iter_mut().rev() {
            let val = scope.get_mut(name);
            if let Some(ref v) = val {
                if v.get_type().is_assignable_to(var_type) {
                    return val;
                }
            }
        }
        None
    }

    pub fn update_vec_at(&mut self, vec_name: &str, index: usize, new_value: VariableValue, vec_type: &VariableType) {
        let vector = self.get_variable_of_type_mut(vec_name, vec_type);
        let Some(vector) = vector else {
            panic!("error: variable named \"{vec_name}\" does not exist");
        };
        let VariableValue::Vec(vector) = vector else {
            panic!("error: tried to index {vec_name} of type {} (expected vector type)", vector.get_type());
        };
        let elem = &mut vector[index];
        let elem_name = elem.get_name().to_string();
        let elem_value = match elem {
            super::Variable::Static(v) => v,
            super::Variable::Named(_, _) => 
                self.get_variable_mut(&elem_name).expect("error: nonexistent member of vector"),
        };
        *elem_value = new_value;
    }
}

#[cfg(test)]
mod tests {
    use image::Rgb;

    use crate::variable::Effect;

    use super::*;

    #[test]
    fn test_push_pop() {
        let mut s = Stack::new();
        assert_eq!(s.scopes.len(), 1);
        assert!(s.top().is_empty());
        s.push();
        assert_eq!(s.scopes.len(), 2);
        assert!(s.pop().is_empty()); // pop
        assert_eq!(s.scopes.len(), 1);
    }

    #[test]
    fn test_add_variable_correct() {
        let mut s = Stack::new();
        s.add_variable("i".to_string(), VariableValue::Int(1));                                     // 1
        s.add_variable("p".to_string(), VariableValue::Pos(1, 1));                                  // (1,1)
        s.add_variable("s".to_string(), VariableValue::String("str".to_string()));                  // "str"
        s.add_variable("c".to_string(), VariableValue::Color(Rgb([255,255,255])));                  // #FFFFFF
        s.add_variable("e".to_string(), VariableValue::Effect(Effect::Blur));                       // blurred
        s.add_variable("a".to_string(), VariableValue::Any(0));                                     // Any(0)
        s.add_variable("vi".to_string(), VariableValue::Vec(vec![VariableValue::Int(1).to_var()])); // [1]

        assert_eq!(s.get_variable("i").unwrap(), &VariableValue::Int(1));
        assert_eq!(s.get_variable("p").unwrap(), &VariableValue::Pos(1, 1));
        assert_eq!(s.get_variable("s").unwrap(), &VariableValue::String("str".to_string()));
        assert_eq!(s.get_variable("c").unwrap(), &VariableValue::Color(Rgb([255,255,255])));
        assert_eq!(s.get_variable("e").unwrap(), &VariableValue::Effect(Effect::Blur));
        assert_eq!(s.get_variable("a").unwrap(), &VariableValue::Any(0));
        assert_eq!(s.get_variable("a").unwrap().get_type().get_binding(), Some(0));
        assert_eq!(s.get_variable("vi").unwrap(), &VariableValue::Vec(vec![VariableValue::Int(1).to_var()]));
    }

    #[test]
    #[should_panic]
    fn test_add_variable_failed() {
        let mut s = Stack::new();
        s.add_variable("a".to_string(), VariableValue::Int(1));
        s.add_variable("a".to_string(), VariableValue::String("this should error".to_string()));
    }

    #[test]
    fn test_update_variable() {
        let mut s = Stack::new();
        s.add_variable("i".to_string(), VariableValue::Int(1));                                     // 1
        s.add_variable("p".to_string(), VariableValue::Pos(1, 1));                                  // (1,1)
        s.add_variable("s".to_string(), VariableValue::String("str".to_string()));                  // "str"
        s.add_variable("c".to_string(), VariableValue::Color(Rgb([255,255,255])));                  // #FFFFFF
        s.add_variable("e".to_string(), VariableValue::Effect(Effect::Blur));                       // blurred
        s.add_variable("a".to_string(), VariableValue::Any(1));                                     // Any(1)
        s.add_variable("vi".to_string(), VariableValue::Vec(vec![VariableValue::Int(1).to_var()])); // [1]

        s.update_variable("i",  VariableValue::Int(2));                                    // 2
        s.update_variable("p",  VariableValue::Pos(2, 2));                                 // (2,2)
        s.update_variable("s",  VariableValue::String("STR".to_string()));                 // "STR"
        s.update_variable("c",  VariableValue::Color(Rgb([0,0,0])));                       // #000000
        s.update_variable("e",  VariableValue::Effect(Effect::Inverse));                   // inversed
        s.update_variable("a",  VariableValue::Any(2));                                    // Any(2)
        s.update_variable("vi", VariableValue::Vec(vec![VariableValue::Int(2).to_var()])); // [2]

        assert_eq!(s.get_variable("i").unwrap(), &VariableValue::Int(2));
        assert_eq!(s.get_variable("p").unwrap(), &VariableValue::Pos(2, 2));
        assert_eq!(s.get_variable("s").unwrap(), &VariableValue::String("STR".to_string()));
        assert_eq!(s.get_variable("c").unwrap(), &VariableValue::Color(Rgb([0,0,0])));
        assert_eq!(s.get_variable("e").unwrap(), &VariableValue::Effect(Effect::Inverse));
        assert_eq!(s.get_variable("a").unwrap(), &VariableValue::Any(2));
        assert_eq!(s.get_variable("a").unwrap().get_type().get_binding(), Some(2));
        assert_eq!(s.get_variable("vi").unwrap(), &VariableValue::Vec(vec![VariableValue::Int(2).to_var()]));
    }

    #[test]
    fn test_get_variable_mut() {
        let mut s = Stack::new();
        s.add_variable("i".to_string(), VariableValue::Int(1));
        let i = s.get_variable_mut("i").unwrap();
        *i = VariableValue::Int(2);
        assert_eq!(s.get_variable("i").unwrap(), &VariableValue::Int(2));
    }

    #[test]
    fn test_get_variable_of_type() {
        let mut s = Stack::new();
        s.add_variable("i".to_string(), VariableValue::Int(1));
        s.push();
        s.add_variable("i".to_string(), VariableValue::String("integer".to_string()));

        let i = s.get_variable_of_type("i", &VariableType::Int).unwrap();
        assert_eq!(i, &VariableValue::Int(1));

        let i = s.get_variable_of_type("i", &VariableType::String).unwrap();
        assert_eq!(i, &VariableValue::String("integer".to_string()));
    }

    #[test]
    fn test_get_variable_of_type_mut() {
        let mut s = Stack::new();
        s.add_variable("i".to_string(), VariableValue::Int(1));
        s.push();
        s.add_variable("i".to_string(), VariableValue::String("integer".to_string()));

        let i = s.get_variable_of_type_mut("i", &VariableType::Int).unwrap();
        *i = VariableValue::Int(2);

        assert_eq!(s.get_variable("i").unwrap(), &VariableValue::String("integer".to_string()));
        s.pop();
        assert_eq!(s.get_variable("i").unwrap(), &VariableValue::Int(2));
    }
}
