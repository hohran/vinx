use crate::{action::Action, event::{Operations, TopLevelOperation}, translator::{StructureTemplate, ast::{self, Ast, AstNode, Range}, automata::Automaton, builtins::{load_builtin_operations, load_builtin_structures, load_top_level_operations}, error::{CompilationError, Location, Warning}, file_manager::FileManager, parser::options::Options}, variable::Stack};

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
    options: Options,
}

impl Parser {
    // Creates a new parser with loaded builtins.
    pub fn new(filepath: &str) -> Result<Self, CompilationError> {
        let mut aut = Automaton::new();
        let mut operations = load_builtin_operations(&mut aut);
        operations.append(&mut load_top_level_operations(&mut aut));
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
            options: Options::default(),
        })
    }

    pub fn parse(&mut self) -> Result<(), CompilationError> {
        let ast = Ast::parse(self.file_manager.current_file());
        for node in &ast.nodes {
            match &node.0 {
                AstNode::Action(a) => self.parse_action(a)?,
                AstNode::Definition(d) => self.parse_definition(d)?,
                AstNode::VarDefinition(d) => self.define_variable(d)?,
                AstNode::Sequence(s) => {
                    let (seq, params) = self.parse_sequence(s)?;
                    let Some(sv) = self.automaton.run(seq.get()) else {
                        return Err(CompilationError::UnknownSequence(seq, self.get_location(&Range::from(s))));
                    };
                    if let Some(top_level_op) = sv.get_top_level_operation(&self.operations) {
                        match top_level_op {
                            TopLevelOperation::LoadFile => {
                                let filepath = params[0].get_value(&self.globals).into_string().to_string();
                                self.parse_file_load(&filepath, &Range::from(s))?;
                            }
                            TopLevelOperation::DoNotSave => {
                                self.options.save_video = false;
                            }
                        }
                    } else {
                        sv.instantiate(params, &self.operations, &self.structures, &mut self.globals);
                    }
                }
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

    fn parse_file_load(&mut self, filepath: &str, range: &ast::Range) -> Result<(), CompilationError> {
        let Some(dependency) = self.file_manager.start(filepath) else {
            // FIXME: when to add ".vinx" to the filepath
            return Err(CompilationError::FileNotFound(filepath.to_string()+".vinx", Some(self.get_location(range))));
        };
        if dependency.is_recursive() {
            let other_file = self.file_manager.current_file().to_string();
            return Err(CompilationError::RecursiveFileDependency(other_file, filepath.to_string()+".vinx", self.get_location(range)));
        }
        if dependency.is_redundant() {
            self.warnings.push(Warning::RedundantFileLoad(filepath.to_string()+".vinx", self.get_location(range)));
            return Ok(());
        }
        self.parse()?;
        self.file_manager.finish_file();
        Ok(())
    }

    pub fn get_location(&self, range: &ast::Range) -> Location {
        Location::new(self.file_manager.current_file(), *range)
    }

    /// Get the top-level stack, list of actions, and defined operations.
    pub fn get(self) -> (Stack,Vec<Action>,Operations,Options) {
        assert_eq!(self._unresolved_parameter_types,0);
        ( self.globals, self.actions, self.operations, self.options )
    }
}

pub fn parse(filepath: &str) -> Result<(Stack,Vec<Action>,Operations,Options), CompilationError> {
    let mut it = Parser::new(filepath)?;
    it.parse()?;
    for w in it.warnings.iter() {
        w.print();
    }
    Ok(it.get())
}
