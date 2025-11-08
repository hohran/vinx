use std::collections::HashMap;

use crate::{action::Action, event::component::Components};

pub struct ComponentClass {
    id: usize,
    parameters: Vec<String>,
    actions: Vec<Action>,
    action_activeness: HashMap<String,bool>,
    components: Components,
}

impl ComponentClass {
    pub fn new(id: usize, parameters: Vec<String>, actions: Vec<Action>, action_activeness: HashMap<String,bool>, components: Components) -> Self {
        Self { id, parameters, actions, action_activeness, components }
    }


}

