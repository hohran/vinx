use tree_sitter::{Node, TreeCursor};

use crate::{action::{Action, Timestamp}, event::{component::Components, Event, Operations}, translator::{seq_to_str, SequenceValue}, variable::{stack::{Stack, VariableMap}, values::VariableValue, Variable}};

use super::{automata::automaton::Automaton, builtins::load_builtin_operations, Word};

#[macro_export]
macro_rules! child {
    ($node:ident[$idx:expr]) => {
        $node.child($idx).expect(&format!("failed to retrieve {}th child of node '{}'", $idx, $node))
    };
}

pub trait Kind {
    fn expect_kind(&self, expect: &str);
}

impl<'a> Kind for Node<'a> {
    fn expect_kind(&self, expect: &str) {
        let kind = self.kind();
        assert_eq!(kind,expect, "error: expected node kind to be {expect}, got {kind} at {:?}", self.range());
    }
}

pub struct InnerTranslator<'a> {
    pub globals: Stack,
    pub components: Components,
    pub actions: Vec<Action>,
    pub source_code: String,
    pub cursor: TreeCursor<'a>,
    pub action_decision_automaton: Automaton,
    pub operations: Operations,
    pub _number_of_builtin_operations: usize,
    pub in_component: bool,
}

impl<'a> InnerTranslator<'a> {
    /// gets string value of node in source code
    pub fn text(&self,node: &Node) -> &str {
        let range = node.range();
        &self.source_code[range.start_byte..range.end_byte]
    }

    /// transforms into owned Translator
    pub fn get(self) -> (Stack, Components,Vec<Action>,Operations) {
        ( self.globals, self.components, self.actions, self.operations ) //, source_code: self.source_code }
    }

    pub fn get_cursor(&'a self) -> TreeCursor<'a> {
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
            if n.kind() != "declaration" {
                continue;
            }
            if n.child_by_field_name("operation").is_some() {
                self.parse_operation(&n);
            }
        }
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
                "declaration" => {
                }
                x => { 
                    panic!("error: unexpected definition: {:?}", x);
                }
            }
        }
    }

    fn get_action_definition(&mut self, node: &Node) {
        let mut i = 0;
        let label = self.get_action_label(node,&mut i);
        let (active,onetime,timestamp,acc) = self.get_action_trigger(&child!(node[i]));
        let events = self.get_action_events(&child!(node[i+1]));
        if !active && label.is_empty() {
            return;     // This action does not have to be processed, since there is no way to activate it
        }
        let a = Action::new(label, active, timestamp, acc, events, onetime);
        self.actions.push(a);
    }

    fn get_action_trigger(&self, node: &Node) -> (bool, bool, Timestamp, Timestamp) {
        let mut i = 0;
        let active = self.get_action_active(&child!(node[i]), &mut i);
        let onetime = !self.get_action_repeats(&child!(node[i]), &mut i);
        let (timestamp,acc) = self.get_action_timestamp_and_acc(&node, &mut i);
        (active,onetime,timestamp,acc)
    }

    fn get_action_label(&self, node: &Node, i: &mut usize) -> String {
        let n = child!(node[*i]);
        if n.kind() == "string" {
            *i += 1;
            self.node_to_string(&n)
        } else {
            "".to_string()
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

    pub fn get_sequence(&self, node: &Node) -> Vec<Word> {
        node.expect_kind("sequence");
        // let mut params = vec![];
        let mut seq = vec![];
        for n in node.named_children(&mut self.get_cursor()) {
            match n.kind() {
                "keyword" => {
                    seq.push(Word::Keyword(self.text(&n).to_string()));
                }
                "value" => {
                    let val = self.get_atomic_value(&n);
                    seq.push(Word::Type(val.get_type()));
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
                    let val = self.get_atomic_value(&n);
                    if let Some(name) = self.get_variable_name(&n) {
                        params.push(Variable::new(name, val.get_type()));
                    } else {
                        params.push(Variable::new_static(val.clone()));
                    }
                    seq.push(Word::Type(val.get_type()));
                }
                ";" => {},
                x => panic!("unexpected type in sequence: {x}")
            }
        }
        // println!("{:?}",seq_to_str(&seq));
        let sv = self.action_decision_automaton.run(&seq).expect(&format!("error: invalid sequence: {}", seq_to_str(&seq)));
        if let SequenceValue::Operation(id) = sv {
                if self.is_builtin_operation(*id) {
                    Event::new(*id, params, vec![], VariableMap::new())
                } else {
                    self.operations.get(id).expect(&format!("error: unknown operation {id}")).instantiate(params)
                }
        } else {
            panic!("unexpected sequence value: {:?}", sv);
        }
    }

    fn get_action_active(&self, node: &Node, i: &mut usize) -> bool {
        if self.text(node) != "!" {
            return true;
        }
        *i += 1;
        return false;
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

    pub fn get_var_definition(&mut self, node: &Node) -> (String,VariableValue) {
        let value = self.get_sequence_value(&child!(node[2]));
        let name = self.text(&node.child(0).unwrap()).to_string();
        println!("assigning {} to {name}",value.to_string());
        self.globals.add_variable(name.clone(), value.clone());
        (name.to_string(),value.clone())
    }
}

pub fn parse(filepath: &str) -> (Stack,Components,Vec<Action>,Operations) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_vinx::LANGUAGE.into()).expect("Could not load vinx grammar");
    let contents = std::fs::read_to_string(filepath).expect("error reading input file");
    let tree = parser.parse(&contents, None).expect("Could not parse input file");
    let root_node = tree.root_node();
    let mut aut = Automaton::new();
    let op_count = load_builtin_operations(&mut aut);
    let mut it = InnerTranslator {
        globals: Stack::new(),
        components: Components::new(),
        actions: vec![],
        source_code: contents,
        cursor: root_node.walk(),
        action_decision_automaton: aut,
        operations: Operations::new(),
        _number_of_builtin_operations: op_count,
        in_component: false,
    };
    it.load(&root_node);
    it.get()
}

