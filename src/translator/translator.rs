use tree_sitter::{Node, TreeCursor};

use crate::{action::Action, event::{component::Components, Operations}, translator::seq_to_str, variable::{stack::Stack, values::VariableValue}};

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
    // pub in_component: bool,
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

    /// load all rules in node: variable definitions, actions, and declarations of operations and (in future) components
    pub fn load(&mut self, node: &Node) {
        for rule in node.children(&mut self.cursor.clone()) {
            match rule.kind() {
                "var_definition" => {
                    self.get_var_definition(&rule);
                }
                "action" => {
                    self.get_action_definition(&rule);
                }
                "declaration" => {
                    if rule.child_by_field_name("operation").is_some() {
                        self.parse_operation(&rule);
                    }
                }
                x => { 
                    panic!("error: unexpected definition: {:?}", x);
                }
            }
        }
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

    pub fn get_var_definition(&mut self, node: &Node) -> (String,VariableValue) {
        let value = self.get_sequence_value(&child!(node[2]));
        let name = self.text(&node.child(0).unwrap()).to_string();
        // println!("assigning {} to {name}",value.to_string());
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
        // in_component: false,
    };
    it.load(&root_node);
    it.get()
}

