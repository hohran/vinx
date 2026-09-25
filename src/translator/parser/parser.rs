use crate::{action::Action, context::Compiletime, event::Operations, translator::{Sequence, SequenceValue, StructureTemplate, ast::{self, Ast, AstNode, Range}, automata::Automaton, builtins::{load_builtin_operations, load_builtin_structures}, error::{CompilationError, Location, Warning}, file_manager::FileManager, parser::CompilationAction}, variable::VariableValue};
use crate::context::Context;

pub struct Parser {
    pub context: Compiletime,
    pub actions: Vec<Action>,
    pub automaton: Automaton,
    pub operations: Operations,
    pub structures: Vec<StructureTemplate>,
    pub file_manager: FileManager,
    pub _unresolved_parameter_types: usize,
    pub self_reference_name: &'static str,
    pub warnings: Vec<Warning>,
}

impl Parser {
    pub fn placeholder() -> Self {
        let automaton = Automaton::new();
        let file_manager = FileManager::placeholder();
        Self::__new(automaton, file_manager, vec![], vec![])
    }

    pub fn __new(automaton: Automaton, file_manager: FileManager, operations: Operations, structures: Vec<StructureTemplate>) -> Self {
        Self { 
            automaton, operations, file_manager, structures, 
            context: Compiletime::new(),
            actions: vec![], 
            _unresolved_parameter_types: 0, 
            self_reference_name: "$self", 
            warnings: vec![] }
    }

    // Creates a new parser with loaded builtins.
    pub fn new(filepath: &str) -> Result<Self, CompilationError> {
        let mut aut = Automaton::new();
        let operations = load_builtin_operations(&mut aut);
        let structures = load_builtin_structures(&mut aut);
        let Some(file_manager) = FileManager::new(filepath) else {
            return Err(CompilationError::FileNotFound(filepath.to_string(), None));
        };
        Ok(Self::__new(aut, file_manager, operations, structures))
        // Ok(Self {
        //     context: Compiletime::new(),
        //     file_manager,
        //     actions: vec![],
        //     automaton: aut,
        //     operations: operations,
        //     structures: builtin_structures,
        //     _unresolved_parameter_types: 0,
        //     self_reference_name: "$self",
        //     warnings: vec![],
        // })
    }

    pub fn get_variable_value(&self, name: &(String, Range)) -> Result<&VariableValue, CompilationError> {
        match self.context.get_variable(&name.0) {
            Some(val) => Ok(val),
            None => Err(CompilationError::UnknownVariableName(name.0.clone(), self.get_location(&name.1))),
        }
    }

    pub fn warn(&mut self, warning: Warning) {
        self.warnings.push(warning);
    }

    // pub fn parse(&mut self) -> Result<(), CompilationError> {
    //     let ast = Ast::parse(self.file_manager.current_file());
    //     for node in &ast.nodes {
    //         match &node.0 {
    //             AstNode::Action(a) => self.parse_action(a)?,
    //             AstNode::Definition(d) => self.parse_definition(d)?,
    //             AstNode::VarDefinition(d) => self.define_variable(d)?,
    //             AstNode::Assignment(a) => self.parse_assignment(a)?,
    //             AstNode::Sequence(s) => {
    //                 for (w,_) in s {
    //                     dbg!(w);
    //                 }
    //                 let (seq, params) = self.parse_sequence(s)?;
    //                 let sv = self.get_sequence_value(&seq)?;
    //                 if let Some(top_level_op) = sv.get_top_level_operation(&self.operations) {
    //                     match top_level_op {
    //                         TopLevelOperation::LoadFile => {
    //                             let filepath = self.context.get_value(&params[0]).into_string().to_string();
    //                             self.parse_file_load(&filepath, &Range::from(s))?;
    //                         }
    //                         TopLevelOperation::DoNotSave => {
    //                             self.context.options.save_video = false;
    //                         }
    //                     }
    //                 } else {
    //                     sv.instantiate(params, &self.operations, &self.structures, &mut self.context);
    //                 }
    //             }
    //         }
    //     }
    //     Ok(())
    // }

    pub fn parse(&mut self) -> Result<(), CompilationError> {
        let ast = Ast::parse(self.file_manager.current_file());
        for node in &ast.nodes {
            match &node {
                AstNode::Action(a) => self.parse_action(a)?,
                AstNode::Definition(d) => self.parse_definition(d)?,
                AstNode::Event(e) => todo!("process event"),
                // AstNode::VarDefinition(d) => self.define_variable(d)?,
                // AstNode::Assignment(a) => self.parse_assignment(a)?,
                // AstNode::Sequence(s) => {
                //     let call = self.parse_call_expression(s)?;
                //     call.value.evaluate_at_compiletime(&call.params, &mut self.context);
                //     // compiletime events can issue `actions` which need to be taken in the main
                //     // compilation process, since they do not hold enough context.
                //     for a in self.context.get_actions() {
                //         match a {
                //             CompilationAction::LoadFile(filepath) => 
                //                 self.parse_file_load(&filepath, &Range::from(s))?
                //         }
                //     }
                // }
            }
        }
        Ok(())
    }

    pub fn get_sequence_value(&self, seq: &Sequence) -> Result<SequenceValue, CompilationError> {
        let Some(sv) = self.automaton.run(seq.get()) else {
            return Err(CompilationError::UnknownSequence(seq.clone()));
        };
        Ok(sv)
    }

    pub fn new_unresolved_variable(&mut self) -> usize {
        self._unresolved_parameter_types += 1;
        return self._unresolved_parameter_types - 1
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
    pub fn get(self) -> (Vec<Action>,Operations,Vec<StructureTemplate>,Compiletime) {
        assert_eq!(self._unresolved_parameter_types,0);
        ( self.actions, self.operations, self.structures, self.context )
    }
}

pub fn parse(filepath: &str) -> Result<(Vec<Action>,Operations,Vec<StructureTemplate>,Compiletime), CompilationError> {
    let mut it = Parser::new(filepath)?;
    it.parse()?;
    for w in it.warnings.iter() {
        w.print();
    }
    Ok(it.get())
}
