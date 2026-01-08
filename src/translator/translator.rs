use tree_sitter::{Node, TreeCursor};

use crate::{action::Action, event::{Operations, component::Components}, translator::{file_manager::{FileDependency, FileManager}, get_children, seq_to_str}, variable::{stack::Stack, values::VariableValue}};

use super::{automata::automaton::Automaton, builtins::load_builtin_operations, Word};

// #[macro_export]
// macro_rules! child {
//     ($node:ident[$idx:expr]) => {
//         $node.child($idx).expect(&format!("failed to retrieve {}th child of node '{}'", $idx, $node))
//     };
// }

pub trait Kind {
    fn expect_kind(&self, expect: &str);
}

impl<'a> Kind for Node<'a> {
    fn expect_kind(&self, expect: &str) {
        let kind = self.kind();
        assert_eq!(kind,expect, "error: expected node kind to be {expect}, got {kind} at {:?}", self.range());
    }
}

pub struct Translator {
    parser: tree_sitter::Parser,
    pub globals: Stack,
    pub components: Components,
    pub actions: Vec<Action>,
    pub action_decision_automaton: Automaton,
    pub operations: Operations,
    pub _number_of_builtin_operations: usize,
    file_manager: FileManager,
    // pub in_component: bool,
}

impl Translator {
    /// gets string value of node in source code
    pub fn text(&self,node: &Node) -> &str {
        let range = node.range();
        let text = self.file_manager.current_file_contents().expect("error: no file is currently processed");
        &text[range.start_byte..range.end_byte]
    }

    /// transforms into owned Translator
    pub fn get(self) -> (Stack, Components,Vec<Action>,Operations) {
        ( self.globals, self.components, self.actions, self.operations ) //, source_code: self.source_code }
    }

    pub fn load_file(&mut self, filepath: &str) {
        let dependency = self.file_manager.start(filepath);
        if dependency.is_recursive() {
            panic!("error: recursive dependency of \"{}\" and \"{filepath}\"", self.file_manager.current_file().unwrap());
        }
        let contents = self.file_manager.current_file_contents().expect(&format!("error: could not read currently loaded file {}", filepath));
        let tree = self.parser.parse(contents, None).expect("Could not parse input file");
        self.load_from_node(&tree.root_node());
        self.file_manager.finish_file();
    }

    /// load all rules in node: variable definitions, actions, and declarations of operations and (in future) components
    pub fn load_from_node(&mut self, node: &Node) {
        for rule in get_children(node) {
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
                "file_load" => {
                    let filepath_node = rule.child_by_field_name("filename").unwrap();
                    self.load_file(&self.node_to_string(&filepath_node));
                }
                x => { 
                    panic!("error: unexpected definition: {:?}", x);
                }
            }
        }
        let seqs = self.action_decision_automaton.get_all_sequences();
        println!("operations:");
        for (seq,sv) in seqs {
            println!("{} -> {:?}",seq_to_str(&seq),sv);
        }
        println!();
    }

    pub fn get_sequence(&self, node: &Node) -> Vec<Word> {
        node.expect_kind("sequence");
        let mut seq = vec![];
        for n in get_children(node) {
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
        node.expect_kind("var_definition");
        let value = self.get_sequence_value(&node.child_by_field_name("rhs").unwrap());
        let name = self.text(&node.child_by_field_name("lhs").unwrap()).to_string();
        // println!("assigning {} to {name}",value.to_string());
        self.globals.add_variable(name.clone(), value.clone());
        (name,value.clone())
    }
}

pub fn parse(filepath: &str) -> (Stack,Components,Vec<Action>,Operations) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_vinx::LANGUAGE.into()).expect("Could not load vinx grammar");
    let mut aut = Automaton::new();
    let op_count = load_builtin_operations(&mut aut);
    let mut it = Translator {
        parser,
        globals: Stack::new(),
        components: Components::new(),
        actions: vec![],
        action_decision_automaton: aut,
        operations: Operations::new(),
        _number_of_builtin_operations: op_count,
        file_manager: FileManager::new(),
        // in_component: false,
    };
    it.load_file(filepath);
    // it.load_from_node(&root_node);
    it.get()
}

