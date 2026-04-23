use std::fmt::Display;

use crate::variable::{VariableValue, Scope, Stack};

/// Instance of a user defined structure
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Structure {
    pub id: usize,
    members: Scope,
}

impl Display for Structure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "structure ({}) {{", self.id)?;
        for (n,v) in &self.members {
            write!(f, "  {n} => {v}")?;
        }
        write!(f, "}}")
    }
}

impl Structure {
    pub fn new(id: usize, members: Scope) -> Self {
        Self { id, members }
    }

    pub fn default(id: usize) -> Self {
        Self { id, members: Scope::new() }
    }

    pub fn copy_members(&self) -> Scope {
        self.members.clone()
    }

    /// Update members' values from the stack
    pub fn update(&mut self, stack: &mut Stack) {
        let member_names: Vec<String> = self.members.iter().map(|(n,_)| n.clone()).collect();
        for n in member_names {
            let v = stack.get_variable(&n).unwrap().clone();
            self.members.insert(n, v);
        }
    }

    pub fn get_members(&self) -> &Scope {
        &self.members
    }

    pub fn get_member(&self, name: &str) -> &VariableValue {
        self.members.get(name).expect(&format!("error: could not find member {name} in structure {}", self.id))
    }

    pub fn get_member_mut(&mut self, name: &str) -> &mut VariableValue {
        self.members.get_mut(name).expect(&format!("error: could not find member {name} in structure {}", self.id))
    }
}
