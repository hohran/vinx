use std::collections::HashMap;

use crate::variable::{VariableLocation, types::VariableType};

use super::values::VariableValue;

pub type VariableMap = HashMap<String,VariableValue>;
#[derive(Debug,Clone)]
pub struct Stack {
    pub layers: Vec<VariableMap>
}

impl Stack {
    pub fn new() -> Self {
        Self { layers: vec![HashMap::new()] }
    }

    pub fn push_layer(&mut self) {
        // println!("=== new layer ===");
        self.layers.push(HashMap::new());
    }

    pub fn push_layer_with(&mut self, layer: VariableMap) {
        // println!("=== new layer ===");
        // for (name,v) in layer.iter() {
        //     println!(" -> {name} ({})",v.get_type().to_string());
        // }
        self.layers.push(layer);
    }

    pub fn pop_layer(&mut self) -> VariableMap{
        assert!(!self.layers.is_empty());
        self.layers.pop().unwrap()
    }

    fn get_top_layer(&mut self) -> &mut VariableMap{
        let top_index = self.layers.len()-1;
        &mut self.layers[top_index]
    }

    pub fn add_variable(&mut self, name: String, value: VariableValue) {
        // println!(" push -> {name} ({})", value.get_type().to_string());
        let top = self.get_top_layer();
        assert!(top.insert(name.clone(), value).is_none(), "error: cannot add variable {name} because it already exists");
    }

    pub fn update_variable(&mut self, name: &str, new_value: VariableValue) {
        for layer_idx in (0..self.layers.len()).rev() {
            let val = self.layers[layer_idx].get_mut(name);
            if let Some(v) = val {
                *v = new_value;
                return;
            }
        }
    }

    pub fn update_vec_at(&mut self, name: &str, index: usize, new_value: VariableValue, var_type: &VariableType) {
        let val = self.get_variable_of_type_mut(name, var_type);
        let Some(val) = val else {
            panic!("error: variable named \"{name}\" does not exist");
        };
        let VariableValue::Vec(v) = val else {
            panic!("error: tried to index {name} of type {} (expected vector type)", val.get_type().to_string());
        };
        let vec_at = &mut v[index];
        let name_at = vec_at.get_name();
        let val_at = match vec_at.location {
            VariableLocation::Scope => {
                self.get_variable_mut(&name_at).expect("error: nonexistent member of vector")
            }
            VariableLocation::Static => {
                vec_at.val.as_mut().expect("error: static variable without value")
            }
            _ => {
                panic!("todo: only scoped and static values are supported in vector assignments");
            }
        };
        // println!(" at vec: {} -> {}", val_at.to_string(), new_value.to_string());
        *val_at = new_value;
    }

    pub fn get_variable(&self, name: &str) -> Option<&VariableValue> {
        for layer in self.layers.iter().rev() {
            let val = layer.get(name);
            if val.is_some() {
                return val;
            }
        }
        None
    }

    pub fn get_variable_of_type(&self, name: &str, var_type: &VariableType) -> Option<&VariableValue> {
        for layer in self.layers.iter().rev() {
            let val = layer.get(name);
            if let Some(v) = val {
                if v.get_type() == *var_type {
                    return val;
                }
            }
        }
        None
    }

    pub fn get_variable_mut(&mut self, name: &str) -> Option<&mut VariableValue> {
        for layer in self.layers.iter_mut().rev() {
            let val = layer.get_mut(name);
            if val.is_some() {
                return val;
            }
        }
        None
    }

    pub fn get_variable_of_type_mut(&mut self, name: &str, var_type: &VariableType) -> Option<&mut VariableValue> {
        for layer in self.layers.iter_mut().rev() {
            let val = layer.get_mut(name);
            if let Some(ref v) = val {
                if v.get_type() == *var_type {
                    return val;
                }
            }
        }
        None
    }

    fn get_variable_vec_mut(&mut self, name: &str) -> Option<&mut VariableValue> {
        for layer in self.layers.iter_mut().rev() {
            let val = layer.get_mut(name);
            if let Some(ref v) = val {
                if matches!(v, VariableValue::Vec(_)) {
                    return val;
                }
            }
        }
        None
    }
}
