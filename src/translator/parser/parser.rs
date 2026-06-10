use crate::{action::Action, event::Operations, translator::{StructureTemplate, ast::{Ast, AstNode}, automata::Automaton, builtins::{load_builtin_operations, load_builtin_structures}, error::{CompilationError, Location, Warning}, file_manager::FileManager, word::Word}, variable::{Stack, Variable}};

pub struct Parser {
    pub globals: Stack,
    pub actions: Vec<Action>,
    pub automaton: Automaton,
    pub operations: Operations,
    pub structures: Vec<StructureTemplate>,
    pub _number_of_builtin_structures: usize,
    pub file_manager: FileManager,
    pub _unresolved_parameter_types: usize,
    pub self_reference_name: &'static str,
    pub warnings: Vec<Warning>,
}

impl Parser {
    // Creates a new parser with loaded builtins.
    pub fn new(filepath: &str) -> Result<Self, CompilationError> {
        let mut aut = Automaton::new();
        let operations = load_builtin_operations(&mut aut);
        let builtin_structures = load_builtin_structures(&mut aut);
        let struct_count = builtin_structures.len();
        let Some(file_manager) = FileManager::new(filepath) else {
            return Err(CompilationError::FileNotFound(filepath.to_string(), None));
        };
        Ok(Self {
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

    pub fn parse(&mut self) -> Result<(), CompilationError> {
        let ast = Ast::parse(self.file_manager.current_file());
        for node in &ast.nodes {
            match node {
                AstNode::Action(a) => self.parse_action(a)?,
                AstNode::Definition(d) => self.parse_definition(d)?,
                AstNode::VarDefinition(d) => self.parse_var_definition(d)?,
                AstNode::FileLoad(f) => self.parse_file_load(f)?,
            }
        }
        Ok(())
    }

    pub fn new_unresolved_variable(&mut self) -> usize {
        self._unresolved_parameter_types += 1;
        self._unresolved_parameter_types - 1
    }

    pub fn resolve_variables(&mut self, count: usize) {
        assert!(self._unresolved_parameter_types >= count);
        self._unresolved_parameter_types -= count;
    }

    fn parse_file_load(&mut self, filepath: &String) -> Result<(), CompilationError> {
        let Some(dependency) = self.file_manager.start(filepath) else {
            // FIXME: when to add ".vinx" to the filepath
            return Err(CompilationError::FileNotFound(filepath.to_string()+".vinx", Some(self.placeholder_location())));
        };
        if dependency.is_recursive() {
            let other_file = self.file_manager.current_file().to_string();
            return Err(CompilationError::RecursiveFileDependency(other_file, filepath.to_string()+".vinx", self.placeholder_location()));
        }
        if dependency.is_redundant() {
            self.warnings.push(Warning::RedundantFileLoad(filepath.to_string()+".vinx", self.placeholder_location()));
            return Ok(());
        }
        self.parse()?;
        self.file_manager.finish_file();
        Ok(())
    }

    pub fn placeholder_location(&self) -> Location {
        let point = tree_sitter::Point { row: 0, column: 0 };
        Location::new("", tree_sitter::Range { start_byte: 0, end_byte: 0, start_point: point, end_point: point })
    }

    /// Get the top-level stack, list of actions, and defined operations.
    pub fn get(self) -> (Stack,Vec<Action>,Operations) {
        assert_eq!(self._unresolved_parameter_types,0);
        ( self.globals, self.actions, self.operations )
    }
}

pub fn parse(filepath: &str) -> Result<(Stack,Vec<Action>,Operations), CompilationError> {
    let mut it = Parser::new(filepath)?;
    it.parse()?;
    for w in it.warnings.iter() {
        w.print();
    }
    Ok(it.get())
}
