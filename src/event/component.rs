use std::collections::HashMap;

// use crate::{action::Action, context::Globals, variable::values::VariableValue};

pub type Components = HashMap<String,Component>;

#[derive(Debug,Clone)]
pub struct Component {
    // class_id: usize,
    // /// name of component type
    // scope: Globals,
    // // operations: HashMap<usize, Vec<Event>>,
    // components: Components,
    // actions: Vec<Action>,
    // action_activeness: HashMap<String, bool>,
}

impl Component {
    // pub fn new(class_id: usize, mut parameters: Vec<String>, mut param_values: Vec<VariableValue>, actions: Vec<Action>, action_activeness: HashMap<String,bool>, components: Components) -> Self {
    //     assert_eq!(parameters.len(), param_values.len());
    //     let mut scope = HashMap::new();
    //     for _ in 0..param_values.len() {
    //         let name = parameters.pop().unwrap();
    //         let val = param_values.pop().unwrap();
    //         scope.insert(name, val);
    //     }
    //     Self { class_id, scope, actions, action_activeness, components }
    // }

    // pub fn step()
}

// impl Component {
    // pub fn empty() -> Self {
    //     Self { scope: HashMap::new(), operations: HashMap::new(), actions: vec![], components: HashMap::new() }
    // }

    // pub fn new_with_attributes(scope: Globals, operations: HashMap<usize,Vec<Event>>, components: Components, actions: Vec<Action>) -> Self {
    //     Self { scope, operations, components, actions }
    // }

    // pub fn add_attribute(&mut self, attr_name: &str, attr_val: VariableValue) {
    //     // if self.metadata.contains_key(attr_name) ...
    //     self.scope.insert(attr_name.to_string(), attr_val);
    // }
    //
    // pub fn add_operation(&mut self, op_id: usize, op: Vec<Event>) {
    //     // if self.operations.contains_key(op_name) ...
    //     self.operations.insert(op_id, op);
    // }
    //
    // pub fn add_action(&mut self, action: Action) {
    //     self.actions.push(action);
    // }

    // pub fn execute_operation(&mut self, op_name: &str, context: &mut Context, parameters: Vec<VariableValue>) {
        // match self.operations.get(op_name) {
        //     Some(op) => {
        //         // add parameters as local variables
        //         let mut i = 1;
        //         for p in parameters {
        //             self.metadata.insert(format!("_arg{i}"), p);
        //             i += 1;
        //         }
        //         // execute commands
        //         for event in op {
        //             if let Event::ActionEvent(e) = event {
        //                 e.process(&mut self.actions);
        //             } else {
        //                 event.process(context, &mut Some(&mut self.metadata), &mut HashMap::new());
        //             }
        //         }
        //         // remove parameters -- this is most probably unnecessary, so it might be removed
        //         // after proper testing ... FIXME
        //         for j in 0..i {
        //             self.metadata.remove(&format!("_arg{j}"));
        //         }
        //     }
        //     None => {
        //         panic!("operation {op_name} does not exist in {}", self.id);
        //     }
        // }
    // }

    // pub fn step(&mut self, _millis: usize, _context: &mut Context) {
    //     return; // TODO
        // for i in 0..self.actions.len() {
        //     let a = &mut self.actions[i];
        //     a.step(millis);
        //     let action_events = a.trigger(context, &mut Some(&mut self.metadata), &mut HashMap::new());
        //     for e in action_events {
        //         e.process(&mut self.actions);
        //     }
        // }
//     }
// }
//
// pub fn rectangle(p1: (usize,usize), p2: (usize,usize), color: Pixel) -> Component {
//     let mut r = Component::new("rectangle");
//     let p1v = Variable::new_local("p1");
//     let p2v = Variable::new_local("p2");
//     let col = Variable::new_local("col");
//     let arg1 = Variable::new_local("_arg1");
//     let arg2 = Variable::new_local("_arg2");
//     r.add_attribute("p1", VariableValue::Pos(p1.0, p1.1));
//     r.add_attribute("p2", VariableValue::Pos(p2.0, p2.1));
//     r.add_attribute("col", VariableValue::Color(color));
//     r.add_operation("move", vec![
//         Event::Move(p1v.clone(), arg1.clone(), arg2.clone()),
//         Event::Move(p2v.clone(), arg1.clone(), arg2.clone()),
//     ]);
//     r.add_operation("movePhase", vec![
//         Event::MovePhase(p1v.clone(), arg1.clone(), arg2.clone()),
//         Event::MovePhase(p2v.clone(), arg1.clone(), arg2.clone()),
//     ]);
//     r.add_action(ActionBuilder::new()
//         .named("draw")
//         .activated_at(crate::action::Timestamp::Frame(1))
//         .with_events(vec![
//             Event::DrawRectPhase(p1v.clone(), p2v.clone(), col.clone())
//         ])
//         .build()
//     );
//     r
// }
//
// pub fn indicator(s: &str) -> Component {
//     let mut c = Component::new("indicator");
//     c.add_action(ActionBuilder::new()
//         .named("indicate")
//         .activated_at(crate::action::Timestamp::Frame(1))
//         .with_events(vec![
//             Event::Print(format!("{s}\n")),
//         ])
//         .build()
//     );
//     c
// }
//
