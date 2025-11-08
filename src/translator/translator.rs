use std::collections::HashMap;

use rsframe::vfx::video::Pixel;
use tree_sitter::{Node, TreeCursor};

use crate::{action::{Action, Timestamp}, context::Globals, event::{component::Components, variable::{types::VariableType, values::{Direction, VariableValue}, Variable}, Event, Operation, Operations}, translator::{automata::linear_automaton::LinearAutomaton, seq_to_str, SequenceValue}};

use super::{automata::automaton::Automaton, builtins::load_builtin_operations, type_inference::type_constraints::TypeConstraints, Word};

macro_rules! child {
    ($node:ident[$idx:expr]) => {
        $node.child($idx).expect(&format!("failed to retrieve {}th child of node '{}'", $idx, $node))
    };
}

pub struct InnerTranslator<'a> {
    globals: HashMap<String,VariableValue>,
    components: Components,
    actions: Vec<Action>,
    source_code: String,
    cursor: TreeCursor<'a>,
    action_decision_automaton: Automaton,
    operations: Operations,
    _number_of_builtin_operations: usize,
    in_component: bool,
}

impl<'a> InnerTranslator<'a> {
    /// gets string value of node in source code
    fn text(&self,node: &Node) -> &str {
        let range = node.range();
        &self.source_code[range.start_byte..range.end_byte]
    }

    /// transforms into owned Translator
    pub fn get(self) -> (Globals,Components,Vec<Action>,Operations) {
        ( self.globals, self.components, self.actions, self.operations ) //, source_code: self.source_code }
    }

    pub fn get_cursor(&self) -> TreeCursor {
        self.cursor.clone()
    }

    pub fn load(&mut self, node: &Node) {
        self.load_declarations(&node);
        self.load_definitions(&node);
        let seqs = self.action_decision_automaton.get_all_sequences();
        println!("operations:");
        for (sv,seq) in seqs {
            println!("{} -> {:?}",seq_to_str(&seq),sv);
        }
        println!();
    }

    /// loads component and operation declarations + their constructors and operations
    pub fn load_declarations(&mut self, node: &Node) {
        for n in node.children(&mut self.cursor.clone()) {
            if n.kind() == "declaration" {
                // parse lhs
                let lhs = n.child_by_field_name("lhs").unwrap();
                let mut seq = vec![];
                let mut operands = vec![];
                let mut var_count = 0;
                for elem in lhs.children(&mut self.cursor.clone()) {
                    match elem.kind() {
                        "keyword" => {
                            seq.push(Word::Keyword(self.text(&elem).to_string()));
                        }
                        "value" => {
                            if let Some(name) = self.get_variable_name(&elem) {
                                operands.push(name.to_string());
                                seq.push(Word::Type(VariableType::Any(var_count)));   // TODO: infer datatype
                                var_count += 1;
                            } else {
                                panic!("error: static values are forbidden in operation definition");
                            }
                        }
                        "label" => {
                            panic!("error: static values are forbidden in operation definition");
                        }
                        x => panic!("error: unexpected type {x} in sequence")
                    }
                }
                if let Some(op) = n.child_by_field_name("operation") {
                    self.handle_operation(&op, &mut operands, &seq);
                } 
                // if let Some(comp) = n.child_by_field_name("component") {
                //     panic!("error: cannot parse components");
                // }
            }
        }
    }

    fn handle_operation(&mut self, events_node: &Node, variables: &mut Vec<String>, signature: &Vec<Word>) -> bool {
        // push variables to scope
        let mut tmp = vec![];
        for i in 0..variables.len() {
            let v = &variables[i];
            let old = self.globals.insert(v.to_string(), VariableValue::Any(i));
            tmp.push(old);
        }
        // get possible event interpretations
        let mut interpretations = vec![]; // store of sequences, and for each
        for e in events_node.named_children(&mut self.get_cursor()) {
            if !self.get_event_interpretations(&e, variables.len(), &mut interpretations) {
                panic!("error: no viable interpretations for operation {:?}", signature);
            }
        }
        // let ints_with_usages: Vec<(&Vec<TypeConstraints>, Vec<bool>)> = interpretations.iter().zip(var_usages).collect();
        if interpretations.len() == 0 {
            // empty declaration
            self.create_typed_operation(signature, variables, &TypeConstraints::new(variables.len()).get_types(), events_node);
            return true;
        }
        self._infer_datatypes_from_interpretations(&interpretations, signature, variables, events_node);
        // revert scope
        for v in variables {
            self.globals.remove(v);
            if let Some(val) = tmp.remove(0) {
                self.globals.insert(v.to_string(), val);
            }
        }
        true
    }

    /// for a sequence, load which variables were used
    /// example:
    ///  seq_node = move $p right by $s
    ///  variables = [ $x, $p, $s ]
    ///  usages > push( F, T, T )
    ///  returned true, because at least one var was set
    fn get_sequence_used_variables(&self, seq_node: &Node, variables: &Vec<String>, usages: &mut Vec<Vec<bool>>) -> bool {
        let mut used = false;
        let mut current_var_usage = vec![false;variables.len()];
        for w in seq_node.named_children(&mut self.get_cursor()) {
            if w.kind() != "value" { continue; }
            if let Some(var_name) = self.get_variable_name(&w) {
                if let Some(used_var) = variables.iter().position(|v| v == var_name) {
                    current_var_usage[used_var] = true;
                    used = true;
                }
            }
        }
        if used {
            usages.push(current_var_usage);
            true
        } else {
            false
        }
    }

    fn add_operation(&mut self, signature: &Vec<Word>, variables: Vec<String>, events: Vec<Event>) {
        let op_id = self._number_of_builtin_operations + self.operations.len();
        println!("adding operation with signature {:?} as {op_id}", signature);
        let mut la = LinearAutomaton::from(signature);
        la.returns(SequenceValue::Operation(op_id));
        self.action_decision_automaton.union(la).unwrap();
        self.operations.insert(op_id, Operation::new(variables, events));
    }

    // fn infer_datatypes_from_interpretations(&mut self, interpretations: &Vec<(&Vec<TypeBounds>,Vec<bool>)>, signature: &Vec<Word>, variables: &Vec<String>, events_node: &Node) {
    //     if interpretations.len() == 0 { return; }
    //     println!("infering types for operation {}", seq_to_str(signature));
    //     self.infer_datatypes_from_interpretations_rec(&TypeBounds::new(variables.len()), &interpretations, signature, variables, events_node);
    // }

    fn _infer_datatypes_from_interpretations(&mut self, interpretations: &Vec<Vec<TypeConstraints>>, signature: &Vec<Word>, variables: &Vec<String>, events_node: &Node) {
        println!("infering datatypes for {:?}", signature);
        println!(" interpretations:");
        for int_pack in interpretations {
            for int in int_pack {
                println!("{}",int);
            }
            println!("---");
        }
        if interpretations.len() == 0 { return; }
        self._infer_datatypes_from_interpretations_rec(&TypeConstraints::new(variables.len()), &interpretations, signature, variables, events_node);
    }

    fn _infer_datatypes_from_interpretations_rec(&mut self, interpretation: &TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Vec<Word>, variables: &Vec<String>, events_node: &Node) {
        if rest.is_empty() { 
            self.create_typed_operation(signature, variables, interpretation.get_types(), events_node);
            return; 
        }
        for int in &rest[0] {
            println!("getting prod {interpretation} + {int}");
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                self._infer_datatypes_from_interpretations_rec(&prod, &rest[1..], signature, variables, events_node);
            }
        }
    }


    /// insert new operation in self.operations
    fn create_typed_operation(&mut self, signature: &Vec<Word>, variables: &Vec<String>, types: &Vec<VariableType>, events_node: &Node) {
        // swap signature
        println!("sign in create_typed_operation: {:?}", signature);
        let mut new_signature = vec![];
        for w in signature {
            if let Word::Type(VariableType::Any(var_id)) = w {
                let v = &types[*var_id];
                new_signature.push(Word::Type(v.clone()));
                self.globals.insert(variables[*var_id].clone(), v.default());
            } else {
                new_signature.push(w.clone());
            }
        }
        // swap event signatures and create events
        let mut op_events = vec![];
        for event in events_node.named_children(&mut self.get_cursor()) {
            let mut params = vec![];
            let mut seq = vec![];
            for w in event.children(&mut self.get_cursor()) {
                match w.kind() {
                    "keyword" => seq.push(Word::Keyword(self.text(&w).to_string())),
                    "label" => {
                        seq.push(Word::Label);
                        params.push(Variable::new_static(VariableValue::Label(self.text(&w).to_string())));
                    }
                    "value" => {
                        let val = self.get_value(&w);
                        seq.push(Word::Type(val.get_type()));
                        if let Some(var_name) = self.get_variable_name(&w) {
                            params.push(Variable::new(var_name));
                        } else {
                            params.push(Variable::new_static(val));
                        }
                    }
                    _ => panic!()
                }
            }
            let sv = self.action_decision_automaton.run(&seq).expect(&format!("error: failed to get sequence {:?}", seq));
            if let SequenceValue::Operation(x) = sv {
                let event = Event::new(*x, params);
                op_events.push(event);
            } else {
                panic!("error: unexpected seq value {:?}", sv);
            }
        }
        self.add_operation(&new_signature, variables.clone(), op_events);
    }

    /// gets all possible interpretations of an event node
    /// interpretation means: for a given sequence, what are the possible types for its ambiguous
    /// variables
    /// for efficiency, it returns whether at least one interpretation was found
    fn get_event_interpretations(&self, node: &Node, var_count: usize, interpretations: &mut Vec<Vec<TypeConstraints>>) -> bool {
        let seq = self.get_sequence(node);
        let ints = self.action_decision_automaton.get_all_paths(&seq, var_count);
        if ints.is_empty() {
            println!("{:?} has no ints",seq);
            return false;
        }
        interpretations.push(ints);
        true
    }

    /// loads specific variables and actions
    pub fn load_definitions(&mut self, node: &Node) {
        for node in node.children(&mut self.cursor.clone()) {
            match node.kind() {
                "var_definition" => {
                    self.get_var_definition(&node);
                }
                "action" => {
                    self.get_action_definition(&node);
                }
                _ => { }
            }
        }
    }

    fn get_action_definition(&mut self, node: &Node) {
        let mut i = 0;
        let label = self.get_action_label(node,&mut i);
        let (active,onetime,timestamp,acc) = self.get_action_trigger(&child!(node[i]));
        let events = self.get_action_events(&child!(node[i+1]));
        let a = Action::new(label.to_string(), active, timestamp, acc, events, onetime);
        self.actions.push(a);
    }

    fn get_action_trigger(&self, node: &Node) -> (bool, bool, Timestamp, Timestamp) {
        let mut i = 0;
        let active = self.get_action_active(&child!(node[i]), &mut i);
        let onetime = !self.get_action_repeats(&child!(node[i]), &mut i);
        let (timestamp,acc) = self.get_action_timestamp_and_acc(&node, &mut i);
        (active,onetime,timestamp,acc)
    }

    fn get_action_label(&self, node: &Node, i: &mut usize) -> &str {
        let n = child!(node[*i]);
        if n.kind() == "label" {
            *i += 1;
            self.text(&n)
        } else {
            ""
        }
    }

    fn get_action_events(&self, node: &Node) -> Vec<Event> {
        assert!(node.kind() == "events", "unexpected type of node {}", node.kind());
        let mut events = vec![];
        if node.child_count() == 1 {
            events.push(self.sequence_to_event(&child!(node[0])));
            return events;
        }
        for e in node.named_children(&mut self.cursor.clone()) {
            events.push(self.sequence_to_event(&e));
        }
        events
    }

    fn get_sequence(&self, node: &Node) -> Vec<Word> {
        assert!(node.kind() == "sequence", "unexpected type of node {}", node.kind());
        // let mut params = vec![];
        let mut seq = vec![];
        for n in node.named_children(&mut self.get_cursor()) {
            match n.kind() {
                "keyword" => {
                    seq.push(Word::Keyword(self.text(&n).to_string()));
                }
                "value" => {
                    let val = self.get_value(&n);
                    seq.push(Word::Type(val.get_type()));
                }
                "label" => {
                    seq.push(Word::Label);
                }
                x => panic!("unexpected type in sequence: {x}")
            }
        }
        seq
    }

    fn sequence_to_event(&self, node: &Node) -> Event {
        assert!(node.kind() == "sequence", "unexpected type of node {}", node.kind());
        let mut params = vec![];
        let mut seq = vec![];
        for n in node.children(&mut self.cursor.clone()) {
            match n.kind() {
                "keyword" => {
                    seq.push(Word::Keyword(self.text(&n).to_string()));
                }
                "value" => {
                    let val = self.get_value(&n);
                    if let Some(name) = self.get_variable_name(&n) {
                        params.push(Variable::new(name));
                    } else {
                        params.push(Variable::new_static(val.clone()));
                    }
                    seq.push(Word::Type(val.get_type()));
                }
                "label" => {
                    let val = Variable::new_static(VariableValue::Label(self.text(&n).to_string()));
                    params.push(val);
                    seq.push(Word::Label);
                }
                ";" => {},
                x => panic!("unexpected type in sequence: {x}")
            }
        }
        println!("{:?}",seq_to_str(&seq));
        let sv = self.action_decision_automaton.run(&seq).expect("error: invalid sequence");
        if let SequenceValue::Operation(id) = sv {
            Event::new(*id, params)
        } else {
            panic!("unexpected sequence value: {:?}", sv);
        }
    }

    fn get_variable_name(&self, node: &Node) -> Option<&str> {
        if node.kind() == "value" {
            if child!(node[0]).kind() != "variable" {
                return None;
            }
            return Some(self.text(node));
        }
        if node.kind() != "variable" {
            return None;
        }
        Some(self.text(node))
    }

    fn get_action_active(&self, _node: &Node, _i: &mut usize) -> bool {
        true    // TODO add a way to have inactive actions
    }

    fn get_action_repeats(&self, node: &Node, i: &mut usize) -> bool {
        *i += 1;
        self.text(node) == "every"
    }

    fn get_action_timestamp_and_acc(&self, node: &Node, i: &mut usize) -> (Timestamp, Timestamp) {
        let q_node = child!(node[*i]);
        let quantifier = if q_node.kind() == "number" { *i += 1; self.node_to_int(&q_node) } else { 1 };
        match self.text(&child!(node[*i])) {
            "frame" | "frames" => (Timestamp::Frame(quantifier),Timestamp::Frame(0)),
            "s" | "second" | "seconds" => (Timestamp::Millis(quantifier*1000),Timestamp::Millis(0)),
            "ms" | "millisecond" | "milliseconds" => (Timestamp::Millis(quantifier),Timestamp::Millis(0)),
            _ => panic!("unexpected time unit {}", self.text(&child!(node[*i])))
        }
    }

    fn get_var_definition(&mut self, node: &Node) {
        let value = self.get_sequence_value(&child!(node[2]));
        let name = self.text(&node.child(0).unwrap());
        println!("assigning {} to {name}",value.to_string());
        // FIXME: when assigning a variable: get its value
        self.globals.insert(name.to_string(), value);
    }

    fn get_sequence_value(&self, node: &Node) -> VariableValue {
        if node.kind() != "sequence" {
            println!("node kind {} is not event", node.kind());
            panic!();
        }
        if node.child_count() == 1 {
            return self.get_value(&child!(node[0]));
        }
        let mut seq = vec![];
        for n in node.children(&mut self.cursor.clone()) {
            match n.kind() {
                "keyword" => {
                    seq.push(Word::Keyword(self.text(&n).to_string()));
                }
                "value" => {
                    let val = self.get_value(&n);
                    seq.push(Word::Type(val.get_type()));
                }
                "label" => {
                    seq.push(Word::Label);
                }
                x => panic!("unexpected type in sequence: {x}")
            }
        }
        let sv = self.action_decision_automaton.run(&seq).expect("error: invalid sequence");
        match sv {
            SequenceValue::Component(id) => VariableValue::Component(*id),
            _ => panic!("error: unexpected sequence value {:?}", sv)
        }
    }

    fn get_value(&self, node: &Node) -> VariableValue {
        assert!(node.kind() == "value", "expected value, got {}", node.kind());
        let val = child!(node[0]);
        match val.kind() {
            "position" => self.pos_to_val(&val),
            "color" => self.color_to_val(&val),
            "variable" => self.globals.get(self.text(&val)).expect(&format!("error: unknown variable {}", self.text(&val))).clone(),
            "direction" => self.direction_to_val(&val),
            "number" => VariableValue::Int(self.node_to_int(&val) as i32),
            "vector" => {
                let mut v = vec![];
                for elem in val.named_children(&mut self.cursor.clone()) {
                    v.push(self.get_value(&elem));
                }
                VariableValue::Vec(v)
            }
            _ => {
                panic!("unknown value type {}", val.kind());
            }
        }
    }

    fn pos_to_val(&self,node: &Node) -> VariableValue {
        if node.kind() != "position" {
            panic!("expected position");
        }
        let x = node.named_child(0).expect("expected position to have 2 child nodes");
        let y = node.named_child(1).expect("expected position to have 2 child nodes");
        VariableValue::Pos(self.node_to_int(&x), self.node_to_int(&y))
    }

    fn direction_to_val(&self, node: &Node) -> VariableValue {
        assert!(node.kind() == "direction", "expected direction, found {}", node.kind());
        match self.text(&node) {
            "left" => VariableValue::Direction(Direction::Left),
            "right" => VariableValue::Direction(Direction::Right),
            "up" => VariableValue::Direction(Direction::Up),
            "down" => VariableValue::Direction(Direction::Down),
            x => panic!("unknown direction type {x}")
        }
    }

    fn color_to_val(&self,node: &Node) -> VariableValue {
        if node.kind() != "color" {
            panic!("expected color");
        }
        let child = child!(node[0]);
        if child.kind() == "color_name" {
            self.color_name_to_val(node)
        } else {
            self.color_code_to_val(node)
        }
    }

    fn color_name_to_val(&self, node: &Node) -> VariableValue {
        match self.text(node) {
            "red" => VariableValue::Color(Pixel::red()),
            "green" => VariableValue::Color(Pixel::green()),
            "blue" => VariableValue::Color(Pixel::blue()),
            "yellow" => VariableValue::Color(Pixel { r: 255, g: 225, b: 53 }),
            "black" => VariableValue::Color(Pixel { r: 0, g: 0, b: 0 }),
            "white" => VariableValue::Color(Pixel { r: 255, g: 255, b: 255 }),
            "orange" => VariableValue::Color(Pixel { r: 255, g: 165, b: 0 }),
            "pink" => VariableValue::Color(Pixel { r: 255, g: 192, b: 203 }),
            "purple" => VariableValue::Color(Pixel { r: 128, g: 0, b: 128 }),
            "brown" => VariableValue::Color(Pixel { r: 165, g: 42, b: 42 }),
            "cyan" => VariableValue::Color(Pixel { r: 0, g: 255, b: 255 }),
            x => {
                panic!("error: unknown color name {}",x);
            }
        }
    }

    fn color_code_to_val(&self, _node: &Node) -> VariableValue {
        panic!("error: color_code_to_val yet to be implemented");
    }

    fn node_to_int(&self, node: &Node) -> usize {
        self.text(&node).parse().expect(&format!("error reading number value of node {}",node.to_string()))
    }
}

pub fn parse(filepath: &str) -> (Globals,Components,Vec<Action>,Operations) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_vinx::LANGUAGE.into()).expect("Could not load vinx grammar");
    let contents = std::fs::read_to_string(filepath).expect("error reading input file");
    let tree = parser.parse(&contents, None).expect("Could not parse input file");
    let root_node = tree.root_node();
    let (builtin_operations,op_count) = load_builtin_operations();
    let mut it = InnerTranslator {
        globals: HashMap::new(),
        components: Components::new(),
        actions: vec![],
        source_code: contents,
        cursor: root_node.walk(),
        action_decision_automaton: builtin_operations,
        operations: Operations::new(),
        _number_of_builtin_operations: op_count,
        in_component: false,
    };
    it.load(&root_node);
    it.get()
}

