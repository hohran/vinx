use std::{collections::HashSet};

use tree_sitter::Node;

use super::*;
use crate::{context::Context, event::Operations, variable::{Scope, Stack, Structure, Variable, VariableType, VariableValue}};

// TODO refactor
#[derive(Debug)]
pub struct StructureTemplate {
    id: usize,
    param_names: Vec<String>,
    param_types: Vec<VariableType>,
    members: Vec<(String, SequenceValue, Vec<Variable>)>,
}

impl StructureTemplate {
    pub fn new(id: usize, param_names: Vec<String>, param_types: Vec<VariableType>, members: Vec<(String, SequenceValue, Vec<Variable>)>) -> Self {
        Self { id, param_names, param_types, members }
    }

    pub fn instantiate(&self, params: Vec<Variable>, context: &mut Context, operations: &Operations, structures: &Vec<StructureTemplate>, stack: &mut Stack) -> Structure {
        assert_eq!(params.len(), self.param_names.len());
        stack.push();
        let mut members = Scope::new();
        for i in 0..params.len() {
            assert!(params[i].get_type().is_assignable_to(&self.param_types[i]));
            members.insert(self.param_names[i].clone(), params[i].get_value(stack).clone());
            stack.add_variable(self.param_names[i].clone(), params[i].get_value(stack).clone()); // TODO: we should cast it to the expected param type (self.params[i])
        }
        for (name,val,ps) in &self.members {
            let member_val = match val {
                SequenceValue::Operation(id) => {
                    operations[*id]
                        .instantiate(ps.clone(), context, operations, structures, stack)
                        .process(context, stack, &mut vec![], operations) // TODO: fix hashmap for action activeness
                        .expect("error: did not have value")
                }
                SequenceValue::Structure(id) => {
                    let val = structures[*id].instantiate(ps.clone(), context, operations, structures, stack);
                    VariableValue::Structure(val)
                }
                SequenceValue::Value(_) => {
                    assert_eq!(ps.len(), 1, "only 1 param for value");
                    ps[0].get_value(stack).clone()
                }
            };
            members.insert(name.clone(), member_val.clone());
            stack.add_variable(name.clone(), member_val);
        }
        stack.pop();
        let s = Structure::new(self.id, members);
        s
    }
}
impl Translator {
    /// Parse structure with signature in `signature_node` and definition in `definition_node`.
    /// Such structure can have multiple interpretations, based on its definition.
    /// For example: `structure of $x` can create interpretations:
    ///     * $x: Color
    ///     * $x: Effect
    /// Every interpretation is saved in the Translator, if it was not already there.
    pub fn parse_structure(&mut self, signature_node: &Node, definition_node: &Node) -> Result<(), CompilationError> {
        let mut signature = self.get_signature(signature_node)?;
        assert!(signature.iterators.is_empty(), "error: iterators not allowed in structure definition: `{signature}`");    // TODO: user friendlify
        let mut method_aut = Automaton::new();
        let (structure_interpretations,methods) = self.get_structure_interpretations(&signature.params, definition_node, &mut method_aut)?;
        for int in structure_interpretations {
            let structure_id = self.structures.len();
            signature.swap_types(int.get_types());
            self.update_stack_with_signature(&signature);
            self.create_typed_structure(&signature, definition_node, structure_id)?;
            self.create_methods(&method_aut, &methods, structure_id, &get_children(definition_node))?;
        }
        Ok(())
    }

    /// Check that members of method have valid names.
    /// They need not to collide with:
    ///  * structure parameters
    ///  * method parameters
    ///  * self reference name
    fn method_member_assert(&self, members: &Vec<String>, operation: &Signature, structure_params: &Vec<String>) -> Result<(), CompilationError> {
        for n in members {
            if structure_params.contains(&n) {
                // return Err(CompilationError::ForbiddenVariableName(n.clone(), ))
            }
            assert!(!structure_params.contains(&n));  // TODO: user friendlify
            assert!(!operation.params.contains(&n));  // TODO: user friendlify
            assert_ne!(n, "$self");  // TODO: user friendlify
        }
        Ok(())
    }

    /// Compute interpretations of structure and return them in HashSet.
    /// Alongside this functionality, all present methods are automatically noted in `method_aut`
    /// and also returned by their signature and id.
    fn get_structure_interpretations(&mut self, operands: &Vec<String>, definition_node: &Node, method_aut: &mut Automaton) -> Result<(HashSet<TypeConstraints>,Vec<(Signature,OperationId)>), CompilationError> {
        let mut methods: Vec<(Signature,usize)> = vec![];
        let mut structure_interpretations = HashSet::new();
        let mut member_names = vec![];
        structure_interpretations.insert(TypeConstraints::new());
        let stmts = get_children(definition_node);
        for i in 0..stmts.len() {
            let stmt = &stmts[i];
            match stmt.kind() {
                "var_definition" => {
                    let (var_id,seq) = self.parse_member(&stmt, &mut member_names, &|_,_| Ok(()))?;
                    self.update_structure_interpretations_with_var(&mut structure_interpretations, var_id, &seq);
                }
                "definition" => {
                    self.globals.push(); {
                        let op_nodes = get_children(&stmt);
                        let op = self.get_signature(&op_nodes[0])?;
                        let (op_members,interpretations) = self.parse_operation_definition(&op_nodes[1], Some(method_aut), &|_,_| Ok(()))?;
                        self.method_member_assert(&op_members, &op, operands)?; // TODO friendlify
                        if interpretations.len() == 0 { println!("nothing for structure"); return Ok((HashSet::new(),vec![])); }   // TODO: warning
                        let constraint_size = operands.len()+member_names.len()+op.params.len();
                        self.update_structure_interpretations(&mut structure_interpretations, &op, interpretations, constraint_size, method_aut, i);
                        self.resolve_variables(op.params.len()+op_members.len());
                        methods.push((op,i));
                    } self.globals.pop();
                }
                x => {
                    panic!("error: structure definition can only contain variable and operation definitions, got {x:?}")
                }
            }
        }
        self.resolve_variables(operands.len()+member_names.len());
        Ok((structure_interpretations,methods))
    }

    /// Create concrete methods for the structure given by `structure_id`.
    /// They are currently stored in `method_aut` for example as:
    ///  - `draw SelfReference at Pos`
    ///
    /// This function makes them bound for given structure in the global automaton:
    ///  - `draw Structure(3) at Pos`
    /// _for `structure_id` = 3_
    fn create_methods(&mut self, method_aut: &Automaton, methods: &Vec<(Signature,usize)>, structure_id: usize, stmts: &Vec<Node<'_>>) -> Result<(), CompilationError> {
        for (seq, sv) in method_aut.get_all_sequences() {
            let SequenceValue::Operation(method_node_id) = sv else { panic!(); };
            let (op,_) = methods.iter().find(|m| m.1 == method_node_id).expect("error: could not find specified node");
            self.globals.push(); {
                let mut signature = op.clone();
                signature.swap_types(&seq.get_types_cloned());
                signature.set_structure_param(structure_id);
                self.push_signature_to_stack(&signature);
                let method_node = &stmts[method_node_id];
                let method_definition = &get_children(method_node)[1];
                let (events, op_members) = self.get_operation_definition(method_definition, Some(structure_id))?;
                self.add_operation(signature, events, op_members);
            } self.globals.pop();
        }
        Ok(())
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
    fn create_typed_structure(&mut self, signature: &Signature, definition_node: &Node, id: usize) -> Result<(), CompilationError> {
        let mut members = vec![];
        for stmt in get_children(definition_node) {
            if stmt.kind() == "var_definition" {
                members.push(self.get_member_definition(&stmt)?);
            }
        }
        if !self.automaton.register(signature.sequence.clone(), SequenceValue::Structure(id)) {
            panic!("error: could not register structure `{signature}` ({})", signature.sequence); // TODO: friendlify
        }
        let structure = StructureTemplate::new(id, signature.params.clone(), signature.sequence.get_types_cloned(), members);
        self.structures.push(structure);
        Ok(())
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
