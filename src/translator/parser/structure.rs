use std::collections::HashSet;

use crate::{translator::{Sequence, SequenceValue, Signature, StructureTemplate, ast, automata::Automaton, error::CompilationError, parser::parser::Parser, sequence::OperationId, type_constraints::TypeConstraints, word::Word}, variable::{VariableType, VariableValue}};

impl Parser {
    /// Parse structure with signature in `signature_node` and definition in `definition_node`.
    /// Such structure can have multiple interpretations, based on its definition.
    /// For example: `structure of $x` can create interpretations:
    ///     * $x: Color
    ///     * $x: Effect
    /// Every interpretation is saved in the Translator, if it was not already there.
    pub fn parse_structure(&mut self, definition: &ast::Definition) -> Result<(), CompilationError> {
        let mut method_aut = Automaton::new();
        let mut signature = self.parse_signature(&definition.signature)?;
        let (structure_interpretations,methods) = self.get_structure_interpretations(&signature.params, definition, &mut method_aut)?;
        for int in structure_interpretations {
            let structure_id = self.structures.len();
            signature.swap_types(int.get_types());
            self.update_stack_with_signature(&signature);
            self.create_typed_structure(&signature, definition, structure_id)?;
            self.create_methods(&method_aut, &methods, structure_id, definition)?;
        }
        Ok(())
    }

    /// Compute interpretations of structure and return them in HashSet.
    /// Alongside this functionality, all present methods are automatically noted in `method_aut`
    /// and also returned by their signature and id.
    fn get_structure_interpretations(&mut self, operands: &Vec<String>, definition: &ast::Definition, method_aut: &mut Automaton) -> Result<(HashSet<TypeConstraints>,Vec<(Signature,OperationId)>), CompilationError> {
        let mut methods: Vec<(Signature,usize)> = vec![];
        let mut structure_interpretations = HashSet::new();
        let mut member_names = vec![];
        structure_interpretations.insert(TypeConstraints::new());
        for i in 0..definition.body.len() {
            let stmt = &definition.body[i];
            match stmt {
                ast::definition::Statement::VarDefinition(var_def) => {
                    let member_id = self.new_unresolved_variable();
                    member_names.push(var_def.name.clone());
                    if !self.globals.add_variable(var_def.name.clone(), VariableValue::Any(member_id)) {
                        return Err(CompilationError::TemporaryError(format!("duplicate member name in operation definition: {}", var_def.name)));
                    }
                    let (seq,_) = self.get_sequence(&var_def.value)?;
                    self.update_structure_interpretations_with_var(&mut structure_interpretations, member_id, &seq);
                }
                ast::definition::Statement::Definition(d) => {
                    self.globals.push(); {
                        let op = self.parse_signature(&d.signature)?;
                        let (op_members,interpretations) = self.parse_operation_definition(&d, Some(method_aut))?;
                        // self.method_member_assert(&op_members, &op, operands)?; // TODO friendlify
                        if interpretations.len() == 0 { println!("nothing for structure"); return Ok((HashSet::new(),vec![])); }   // TODO: warning
                        let constraint_size = operands.len()+member_names.len()+op.params.len();
                        self.update_structure_interpretations(&mut structure_interpretations, &op, interpretations, constraint_size, method_aut, i);
                        self.resolve_variables(op.params.len()+op_members.len());
                        methods.push((op,i));
                    } self.globals.pop();
                }
                ast::definition::Statement::Event(_) => panic!("error: events are not possible in structure definitions"),
            }
        }
        self.resolve_variables(operands.len()+member_names.len());
        Ok((structure_interpretations,methods))
    }

    /// Refine structure interpretations to match `method` with `events_interpretations`.
    /// Interpretations of used methods are also evaluated and stored in `aut` with `note_method`.
    fn update_structure_interpretations(&mut self, structure_interpretations: &mut HashSet<TypeConstraints>, method: &Signature, events_interpretations: Vec<Vec<TypeConstraints>>, constraint_size: usize, aut: &mut Automaton, method_family: usize) {
        let mut new_structure_ints = HashSet::new();
        for mut int in structure_interpretations.drain() {
            int.resize_to(constraint_size);
            let new_ints = self.infer_method_types_rec(int, &events_interpretations, &method, aut, method_family);
            for i in new_ints {
                new_structure_ints.insert(i);
            }
        }
        *structure_interpretations = new_structure_ints;
    }

    /// Recursively infer the interpretations of method given by `signature` and note them in `aut`.
    /// `method_family` is a unique identifier of the method, shared between its interpretations.
    fn infer_method_types_rec(&mut self, interpretation: TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Signature, aut: &mut Automaton, method_family: usize) -> Vec<TypeConstraints> {
        if rest.is_empty() { 
            let type_cutoff = interpretation.get_types().len() - signature.params.len(); // only include structure params
            self.note_method(&signature, interpretation.clone(), aut, method_family);
            return vec![interpretation.cut_to(type_cutoff)]; 
        }
        let mut types = vec![];
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                types.append(&mut self.infer_method_types_rec(prod, &rest[1..], signature, aut, method_family));
            }
        }
        types
    }

    /// Refine structure interpretations for a variable definition.
    fn update_structure_interpretations_with_var(&mut self, structure_interpretations: &mut HashSet<TypeConstraints>, var_id: usize, rhs: &Sequence) {
        let mut new_structure_ints = HashSet::new();
        for struct_int in structure_interpretations.drain() {
            for var_int in self.automaton.get_interpretations(rhs.get(), Some(var_id), &self.operations) {
                if let Some(new_int) = struct_int.clone().intersect(var_int) {
                    new_structure_ints.insert(new_int);
                }
            }
        }
        *structure_interpretations = new_structure_ints;
    }

    /// Create new structure with `id`.
    /// This means it is stored in global structure list, and its signature is registered in the
    /// global automaton.
    /// It is important that its signature has set the desired types.
    fn create_typed_structure(&mut self, signature: &Signature, definition: &ast::Definition, id: usize) -> Result<(), CompilationError> {
        let mut members = vec![];
        for stmt in &definition.body {
            if let ast::definition::Statement::VarDefinition(var_def) = stmt {
                members.push(self.get_member_definition(var_def)?);
            }
        }
        if !self.automaton.register(signature.sequence.clone(), SequenceValue::Structure(id)) {
            panic!("error: could not register structure `{signature}` ({})", signature.sequence); // TODO: friendlify
        }
        let structure = StructureTemplate::new(id, signature.params.clone(), signature.sequence.get_types_cloned(), members);
        self.structures.push(structure);
        Ok(())
    }

    /// Create concrete methods for the structure given by `structure_id`.
    /// They are currently stored in `method_aut` for example as:
    ///  - `draw SelfReference at Pos`
    ///
    /// This function makes them bound for given structure in the global automaton:
    ///  - `draw Structure(3) at Pos`
    /// _for `structure_id` = 3_
    fn create_methods(&mut self, method_aut: &Automaton, methods: &Vec<(Signature,usize)>, structure_id: usize, definition: &ast::Definition) -> Result<(), CompilationError> {
        for (seq, sv) in method_aut.get_all_sequences() {
            let SequenceValue::Operation(method_id) = sv else { panic!(); };
            let (op,_) = methods.iter().find(|m| m.1 == method_id).expect("error: could not find specified node");
            self.globals.push(); {
                let mut signature = op.clone();
                signature.swap_types(&seq.get_types_cloned());
                signature.set_structure_param(structure_id);
                self.push_signature_to_stack(&signature);
                let ast::definition::Statement::Definition(method) = &definition.body[method_id] else {
                    panic!("error: is not mehod"); // TODO: make nicer
                };
                let (events, op_members) = self.get_operation_definition(method, Some(structure_id))?;
                self.add_operation(signature, events, op_members);
            } self.globals.pop();
        }
        Ok(())
    }

    /// Push every `signature` parameter to the stack, assigning it the default value of its type.
    pub fn push_signature_to_stack(&mut self, signature: &Signature) {
        signature.foreach(|p,t| if !self.globals.add_variable(p.to_string(), t.default()) { panic!("error: unexpected redeclaration of variables") } );
    }

    /// Register symbolic method in `aut` with id equal to the `method_family`.
    ///
    /// For example, consider a method with signature: `draw $c $self`
    /// This function could note `draw Color SelfReference` with some method_family M
    /// If this method had a second interpretation `draw Effect SelfReference`, its family would
    /// also be M.
    ///
    /// Usually, `method_family` is 0 for the first defined method, 1 for the next one, and so on.
    fn note_method(&self, signature: &Signature, interpretation: TypeConstraints, aut: &mut Automaton, method_family: usize) {
        let mut new_signature = vec![];
        let types = interpretation.get_types();
        for w in signature.sequence.get() {
            if let Word::Type(VariableType::Any(var_id)) = w {
                let v = if types.len() > *var_id { &types[*var_id] } else { &VariableType::Any(*var_id) };
                if signature.iterators.contains(var_id) {
                    new_signature.push(Word::Type(VariableType::Vec(Box::new(v.clone()))));
                } else {
                    new_signature.push(Word::Type(v.clone()));
                }
            } else {
                new_signature.push(w.clone());
            }
        }
        let op_id = method_family;
        aut.register(Sequence::from(new_signature), SequenceValue::Operation(op_id));
    }
}
