use std::collections::HashSet;
use crate::context::Context;

use crate::translator::error::Warning;
use crate::{translator::{Sequence, SequenceValue, Signature, StructureTemplate, ast, automata::Automaton, error::CompilationError, parser::parser::Parser, sequence::{OperationId, SequenceType}, type_constraints::TypeConstraints, word::Word}, variable::{Variable, VariableType}};

pub type StructureMember = (String, SequenceValue, Vec<Variable>); // name, uninitalized value, parameters (used for initialization of the value)

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
        if structure_interpretations.is_empty() {
            self.warn(Warning::StructureWithoutInterpretation(signature.clone()));
        }
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
        let mut method_count = 0;
        for stmt in &definition.body {
            match &stmt.0 {
                ast::definition::Statement::VarDefinition(var_def) => {
                    let member_id = self.new_unresolved_variable();
                    let member = self.get_var_definition(var_def, Some(member_id))?;
                    let name = member.get_name().to_string();
                    member_names.push(name.clone());
                    if !self.context.add_variable(name.clone(), member.get_type().default()) {
                        return Err(CompilationError::TemporaryError(format!("duplicate member name in operation definition: {}", name)));
                    }
                    let (seq, _) = member.get_value();
                    self.update_structure_interpretations_with_var(&mut structure_interpretations, member.get_type(), seq);
                }
                ast::definition::Statement::Definition(d) => {
                    self.context.push_scope(); {
                        let op = self.parse_signature(&d.signature)?;
                        let (op_members,interpretations) = self.parse_operation_definition(&d, Some(method_aut))?;
                        let constraint_size = operands.len()+member_names.len()+op.params.len();
                        // this also registers the method
                        self.update_structure_interpretations(&mut structure_interpretations, &op, interpretations, constraint_size, method_aut);
                        self.resolve_variables(op.params.len()+op_members.len());
                        methods.push((op,method_count));
                        method_count += 1;
                    } self.context.pop_scope();
                }
                ast::definition::Statement::Assignment(_) => panic!("error: assignments are not possible in structure definitions"), // TODO: friendlify
                ast::definition::Statement::Event(_) => panic!("error: events are not possible in structure definitions"),
            }
        }
        self.resolve_variables(operands.len()+member_names.len());
        Ok((structure_interpretations,methods))
    }

    /// Refine structure interpretations to match `method` with `events_interpretations`.
    /// Interpretations of used methods are also evaluated and stored in `aut` with `note_method`.
    fn update_structure_interpretations(&mut self, structure_interpretations: &mut HashSet<TypeConstraints>, method: &Signature, events_interpretations: Vec<Vec<TypeConstraints>>, constraint_size: usize, aut: &mut Automaton) {
        let mut new_structure_ints = HashSet::new();
        if events_interpretations.is_empty() {
            *structure_interpretations = new_structure_ints;
            return
        }
        for mut int in structure_interpretations.drain() {
            int.resize_to(constraint_size);
            let new_ints = self.infer_method_types_rec(int, &events_interpretations, &method, aut);
            for i in new_ints {
                new_structure_ints.insert(i);
            }
        }
        *structure_interpretations = new_structure_ints;
    }

    /// Recursively infer the interpretations of method given by `signature` and note them in `aut`.
    /// `method_family` is a unique identifier of the method, shared between its interpretations.
    fn infer_method_types_rec(&mut self, interpretation: TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Signature, aut: &mut Automaton) -> Vec<TypeConstraints> {
        if rest.is_empty() { 
            let type_cutoff = interpretation.get_types().len() - signature.params.len(); // only include structure params
            self.note_method(&signature, interpretation.clone(), aut);
            return vec![interpretation.cut_to(type_cutoff)]; 
        }
        let mut types = vec![];
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                types.append(&mut self.infer_method_types_rec(prod, &rest[1..], signature, aut));
            }
        }
        types
    }

    /// Refine structure interpretations for a variable definition.
    fn update_structure_interpretations_with_var(&mut self, structure_interpretations: &mut HashSet<TypeConstraints>, var_type: &VariableType, rhs: &Sequence) {
        let mut new_structure_ints = HashSet::new();
        for struct_int in structure_interpretations.drain() {
            for var_int in self.automaton.get_interpretations(rhs.get(), Some(var_type), &self.operations) {
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
            if let ast::definition::Statement::VarDefinition(var_def) = &stmt.0 {
                members.push(self.get_member_definition(var_def)?);
            }
        }
        if !self.automaton.register(signature.sequence.clone(), SequenceType::Structure) {
            self.warn(Warning::ExistingSignature(signature.clone())); // this warning is maybe not needed
        } else {
            let structure = StructureTemplate::new(id, signature.params.clone(), signature.sequence.get_types_cloned(), members);
            self.structures.push(structure);
        }
        Ok(())
    }

    /// Get a member definition from an operation statement (e.g., $member = something).
    pub fn get_member_definition(&mut self, var_def: &ast::VarDefinition) -> Result<StructureMember, CompilationError> {
        let definition = self.get_var_definition(var_def, None)?;
        let name = definition.get_name().clone();
        let (seq, params) = definition.get_value();
        let sv = self.get_sequence_value(seq)?;
        let seq_type = sv.get_concrete_type(&self.operations, params);
        if !seq_type.is_assignable_to(definition.get_type()) {
            panic!("error: type {seq_type} does not match declared type {}", definition.get_type()); // TODO friendlify
        }
        self.context.update_variable(&name, seq_type.default());
        return Ok((name, sv, params.clone()));
        //
        // //
        // let member_name = &var_def.name.0;
        // if let Some(t) = &var_def.typ {
        //     let typ = self.parse_type(&t.0)?;
        //     if let Some(val) = &var_def.value {
        //         let (seq,params) = self.parse_sequence(&val.0)?;
        //         let sv = self.get_sequence_value(&seq)?;
        //         let seq_type = sv.into_type(&self.operations);
        //         if !seq_type.is_assignable_to(&typ) {
        //             panic!("error: type {seq_type} does not match declared type {typ}"); // TODO friendlify
        //         }
        //         self.context.update_variable(member_name, seq_type.default());
        //         Ok((member_name.clone(),sv.clone(),params))
        //     } else {
        //         self.context.update_variable(member_name, typ.default());
        //         // FIXME: this expects that operation 0 is the single value return -v
        //         Ok((member_name.clone(),SequenceValue::Operation(0),vec![typ.default().to_var()]))
        //     }
        // } else {
        //     let Some(val) = &var_def.value else {
        //         panic!("error: variable definition needs to have at least one of [type, value] specified");
        //     };
        //     let (seq,params) = self.parse_sequence(&val.0)?;
        //     let sv = self.get_sequence_value(&seq)?;
        //     self.context.update_variable(member_name, sv.into_type(&self.operations).default());
        //     Ok((member_name.clone(),sv.clone(),params))
        // }
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
            let Some((op,_)) = methods.iter().find(|m| m.1 == method_id) else {
                panic!();
            };
            self.context.push_scope(); {
                let mut signature = op.clone();
                signature.swap_types(&seq.get_types_cloned());
                signature.set_structure_param(structure_id)?;
                self.push_signature_to_stack(&signature);
                let method = definition.find_nth_method(method_id);
                let member_names = self.get_operation_members(method)?;
                let events = self.get_operation_definition(method, Some(structure_id))?;
                self.add_operation(signature, events, self.get_members(&member_names));
            } self.context.pop_scope();
        }
        Ok(())
    }

    /// Push every `signature` parameter to the stack, assigning it the default value of its type.
    pub fn push_signature_to_stack(&mut self, signature: &Signature) {
        signature.foreach(|p,t| if !self.context.add_variable(p.to_string(), t.default()) { panic!("error: unexpected redeclaration of variables") } );
    }

    /// Register symbolic method in `aut` with id equal to the `method_family`.
    ///
    /// For example, consider a method with signature: `draw $c $self`
    /// This function could note `draw Color SelfReference` with some method_family M
    /// If this method had a second interpretation `draw Effect SelfReference`, its family would
    /// also be M.
    ///
    /// Usually, `method_family` is 0 for the first defined method, 1 for the next one, and so on.
    fn note_method(&self, signature: &Signature, interpretation: TypeConstraints, aut: &mut Automaton) {
        let mut new_signature = Sequence::new(signature.get_location().clone());
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
        aut.register(new_signature, SequenceType::Operation);
    }
}
