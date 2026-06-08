use tree_sitter::Node;

use super::*;
use crate::{action::Action, event::Operations, variable::{Stack, VariableValue}};

pub struct Translator {
    parser: tree_sitter::Parser,
    pub globals: Stack,
    pub actions: Vec<Action>,
    pub automaton: Automaton,
    pub operations: Operations,
    pub structures: Vec<StructureTemplate>,
    pub _number_of_builtin_structures: usize,
    pub file_manager: FileManager,
    pub _unresolved_parameter_types: usize,
    pub self_reference_name: &'static str,
    warnings: Vec<Warning>,
}

impl Translator {
    // Creates a new translator with loaded builtins.
    pub fn new(filepath: &str) -> Result<Self, CompilationError> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_vinx::LANGUAGE.into()).expect("error: could not load vinx grammar");
        let mut aut = Automaton::new();
        let operations = load_builtin_operations(&mut aut);
        let builtin_structures = load_builtin_structures(&mut aut);
        let struct_count = builtin_structures.len();
        let Some(file_manager) = FileManager::new(filepath) else {
            return Err(CompilationError::FileNotFound(filepath.to_string(), None));
        };
        Ok(Translator {
            parser,
            file_manager,
            globals: Stack::new(),
            actions: vec![],
            automaton: aut,
            operations: operations,
            structures: builtin_structures,
            _number_of_builtin_structures: struct_count,
            _unresolved_parameter_types: 0,
            self_reference_name: "$self",
            warnings: vec![],
        })
    }

    pub fn get_location(&self, node: &Node) -> Location {
        return Location::new(self.file_manager.current_file(), node.range())
    }

    pub fn expect_node_kind(&self, node: &Node, expect: &str) {
        let kind = node.kind();
        if kind != expect {
            let file = self.file_manager.current_file().to_string();
            let start = node.range().start_point;
            panic!("{file}:{}:{}: error: expected node type to be {}, got {}", start.row, start.column, expect, kind);
        }
    }

    /// Get the top-level stack, list of actions, and defined operations.
    pub fn get(self) -> (Stack,Vec<Action>,Operations) {
        assert_eq!(self._unresolved_parameter_types,0);
        ( self.globals, self.actions, self.operations )
    }

    // Load the source code in file, specified by `node`.
    //
    // If the file is not found, or is recursive with another loaded file, return an error.
    fn load_file(&mut self, node: &Node) -> Result<(), CompilationError> {
        self.expect_node_kind(node, "file_load");
        let filepath_node = node.child_by_field_name("filename").unwrap();
        let filepath = &self.node_to_string(&filepath_node);
        let Some(dependency) = self.file_manager.start(filepath) else {
            // FIXME: when to add ".vinx" to the filepath
            return Err(CompilationError::FileNotFound(filepath.to_string()+".vinx", Some(self.get_location(node))));
        };
        if dependency.is_recursive() {
            let other_file = self.file_manager.current_file().to_string();
            return Err(CompilationError::RecursiveFileDependency(other_file, filepath.to_string()+".vinx", self.get_location(node)));
        }
        if dependency.is_redundant() {
            self.warnings.push(Warning::RedundantFileLoad(filepath.to_string()+".vinx", self.get_location(node)));
            return Ok(());
        }
        let contents = self.file_manager.current_file_contents();
        let tree = self.parser.parse(contents, None).expect("Could not parse input file");
        self.load_from_node(&tree.root_node())?;
        self.file_manager.finish_file();
        Ok(())
    }

    // Compile the specified program.
    pub fn compile(&mut self) -> Result<(), CompilationError> {
        let contents = self.file_manager.load_file_contents();
        let tree = self.parser.parse(contents, None).expect("Could not parse input file");
        self.load_from_node(&tree.root_node())?;
        self.file_manager.finish_file();
        Ok(())
    }

    /// Parse and create given operation/structure based on the definition
    fn handle_definition(&mut self, node: &Node) -> Result<(), CompilationError> {
        self.expect_node_kind(node, "definition");
        let children = get_children(node);
        let signature = &children[0];
        let definition = &children[1];
        // if definition contains operation definitions, it is structure
        let structure_proof = get_children(definition).iter().find(|n| n.kind() == "definition").cloned();
        let operation_proof = get_children(definition).iter().find(|n| n.kind() == "sequence").cloned();
        if structure_proof.is_some() && operation_proof.is_some() {
            return Err(CompilationError::VagueDefinition(
                    self.get_location(signature),
                    self.get_location(&operation_proof.unwrap()), 
                    self.get_location(&structure_proof.unwrap())))
        }
        self.globals.push(); {
            if structure_proof.is_some() {
                self.parse_structure(signature, definition)?;
            } else {
                self.parse_operation(signature, definition)?;
            }
        } self.globals.pop();
        Ok(())
    }

    /// Loads all statements of the `root_node` of given file.
    pub fn load_from_node(&mut self, root_node: &Node) -> Result<(), CompilationError> {
        self.expect_node_kind(root_node, "source_file");
        for stmt in get_children(root_node) {
            match stmt.kind() {
                "var_definition" => {
                    self.get_var_definition(&stmt)?;
                }
                "action" => {
                    self.get_action_definition(&stmt)?;
                }
                "definition" => {
                    self.handle_definition(&stmt)?;
                }
                "file_load" => {
                    self.load_file(&stmt)?;
                }
                x => { 
                    panic!("error: unexpected statement: {:?}", x);
                }
            }
        }
        Ok(())
    }

    pub fn get_var_definition_name(&self, node: &Node) -> &str {
        self.expect_node_kind(node, "var_definition");
        self.text(&node.child_by_field_name("lhs").unwrap())
    }

    pub fn get_var_definition(&mut self, node: &Node) -> Result<(String,VariableValue), CompilationError> {
        self.expect_node_kind(node, "var_definition");
        let value = self.get_sequence_value(&node.child_by_field_name("rhs").unwrap())?;
        let name = self.text(&node.child_by_field_name("lhs").unwrap()).to_string();
        if self.is_forbidden_variable_name(&name) {
            return Err(CompilationError::ForbiddenVariableName(name, self.get_location(node)));
        }
        if self.globals.add_variable(name.clone(), value.clone()) {
            Ok((name,value.clone()))
        } else {
            Err(CompilationError::RedeclaredVariable(name, self.get_location(&node.child_by_field_name("lhs").unwrap())))
        }
    }

    fn is_forbidden_variable_name(&self, name: &str) -> bool {
        name == self.self_reference_name
    }

    pub fn new_unresolved_variable(&mut self) -> usize {
        self._unresolved_parameter_types += 1;
        self._unresolved_parameter_types - 1
    }

    pub fn resolve_variables(&mut self, count: usize) {
        assert!(self._unresolved_parameter_types >= count);
        self._unresolved_parameter_types -= count;
    }

}

pub fn parse(filepath: &str) -> Result<(Stack,Vec<Action>,Operations), CompilationError> {
    let mut it = Translator::new(filepath)?;
    it.compile()?;
    for w in it.warnings.iter() {
        w.print();
    }
    Ok(it.get())
}

