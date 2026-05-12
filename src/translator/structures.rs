use std::collections::HashSet;

use tree_sitter::Node;

use crate::{translator::{OperationId, SequenceValue, StructureTemplate, automata::Automaton, get_children, sequence::Sequence, signature::Signature, type_constraints::TypeConstraints}, variable::{VariableType, VariableValue}};

use super::{translator::Translator, Word};

impl Translator {
    /// Parse structure with signature in `signature_node` and definition in `definition_node`.
    /// Such structure can have multiple interpretations, based on its definition.
    /// For example: `structure of $x` can create interpretations:
    ///     * $x: Color
    ///     * $x: Effect
    /// Every interpretation is saved in the Translator, if it was not already there.
    pub fn parse_structure(&mut self, signature_node: &Node, definition_node: &Node) {
        let mut signature = self.parse_signature(signature_node);
        assert!(signature.iterators.is_empty(), "error: iterators not allowed in structure definition: `{signature}`");    // TODO: user friendlify
        let mut method_aut = Automaton::new();
        let (structure_interpretations,methods) = self.get_structure_interpretations(&signature.params, definition_node, &mut method_aut);
        for int in structure_interpretations {
            let structure_id = self.structures.len();
            signature.swap_types(int.get_types());
            self.update_stack_with_signature(&signature);
            self.create_typed_structure(&signature, definition_node, structure_id);
            self.create_methods(&method_aut, &methods, structure_id, &get_children(definition_node));
        }
    }

    /// Check that members of method have valid names.
    /// They need not to collide with:
    ///  * structure parameters
    ///  * method parameters
    ///  * self reference name
    fn method_member_assert(&self, members: &Vec<String>, operation: &Signature, structure_params: &Vec<String>) -> bool {
        for n in members {
            assert!(!structure_params.contains(&n));  // TODO: user friendlify
            assert!(!operation.params.contains(&n));  // TODO: user friendlify
            assert_ne!(n, "$self");  // TODO: user friendlify
        }
        true
    }

    /// Compute interpretations of structure and return them in HashSet.
    /// Alongside this functionality, all present methods are automatically noted in `method_aut`
    /// and also returned by their signature and id.
    fn get_structure_interpretations(&mut self, operands: &Vec<String>, definition_node: &Node, method_aut: &mut Automaton) -> (HashSet<TypeConstraints>,Vec<(Signature,OperationId)>) {
        let mut methods: Vec<(Signature,usize)> = vec![];
        let mut structure_interpretations = HashSet::new();
        let mut member_names = vec![];
        structure_interpretations.insert(TypeConstraints::new());
        let stmts = get_children(definition_node);
        for i in 0..stmts.len() {
            let stmt = &stmts[i];
            match stmt.kind() {
                "var_definition" => {
                    let (var_id,seq) = self.add_member(&stmt, &mut member_names);
                    structure_interpretations = self.update_structure_interpretations_with_var(structure_interpretations, var_id, &seq);
                }
                "definition" => {
                    self.globals.push();
                    let op_nodes = get_children(&stmt);
                    let op = self.parse_signature(&op_nodes[0]);
                    let (op_members,interpretations) = self.parse_operation_definition(&op_nodes[1], &op.sequence, Some(method_aut));
                    assert!(self.method_member_assert(&op_members, &op, operands)); // TODO friendlify
                    if interpretations.len() == 0 { println!("nothing for structure"); return (HashSet::new(),vec![]); }   // TODO: warning
                    let constraint_size = operands.len()+member_names.len()+op.params.len();
                    structure_interpretations = self.update_structure_interpretations(structure_interpretations, &op, interpretations, constraint_size, method_aut, i);
                    self.resolve_variables(op.params.len()+op_members.len());
                    methods.push((op,i));
                    self.globals.pop();
                }
                x => {
                    panic!("error: structure definition can only contain variable and operation definitions, got {x:?}")
                }
            }
        }
        self.resolve_variables(operands.len()+member_names.len());
        (structure_interpretations,methods)
    }

    fn create_methods(&mut self, method_aut: &Automaton, methods: &Vec<(Signature,usize)>, structure_id: usize, stmts: &Vec<Node<'_>>) {
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
                let (events, op_members) = self.get_operation_definition(method_definition, Some(structure_id));
                self.add_operation(signature, events, op_members);
            } self.globals.pop();
        }
    }

    fn update_structure_interpretations(&mut self, structure_interpretations: HashSet<TypeConstraints>, method: &Signature, method_interpretations: Vec<Vec<TypeConstraints>>, constraint_size: usize, aut: &mut Automaton, method_family: usize) -> HashSet<TypeConstraints> {
        let mut new_structure_ints = HashSet::new();
        for mut int in structure_interpretations {
            int.resize_to(constraint_size);
            let new_ints = self.infer_method_types_rec(&int, &method_interpretations, &method, aut, method_family);
            for i in new_ints {
                new_structure_ints.insert(i);
            }
        }
        new_structure_ints
    }

    fn update_structure_interpretations_with_var(&mut self, structure_interpretations: HashSet<TypeConstraints>, var_id: usize, rhs: &Sequence) -> HashSet<TypeConstraints> {
        let mut new_structure_ints = HashSet::new();
        for mut struct_int in structure_interpretations {
            if rhs.len() == 1 && let Some(t) = rhs.at(0).get_type() {
                let ret = struct_int.intersect_var(var_id, t);
                assert!(ret);
                new_structure_ints.insert(struct_int);
                continue;
            }
            for var_int in self.action_decision_automaton.get_interpretations(rhs.get(), Some(var_id), &self.operations) {
                if let Some(new_int) = struct_int.clone().intersect(var_int) {
                    new_structure_ints.insert(new_int);
                }
            }
        }
        new_structure_ints
    }

    /// insert new structure
    fn create_typed_structure(&mut self, signature: &Signature, node: &Node, id: usize) {
        let mut members = vec![];
        for stmt in get_children(node) {
            if stmt.kind() != "var_definition" {
                continue;
            }
            members.push(self.get_member(&stmt));
        }
        // println!("adding structure {id} with signature \"{}\"", Sequence::from(new_signature.clone()));
        if !self.action_decision_automaton.register(signature.sequence.clone(), SequenceValue::Structure(id)) {
            return;
        }
        let structure = StructureTemplate::new(id, signature.params.clone(), signature.sequence.get_types_cloned(), members);
        self.structures.push(structure);
    }

    fn infer_method_types_rec(&mut self, interpretation: &TypeConstraints, rest: &[Vec<TypeConstraints>], signature: &Signature, aut: &mut Automaton, method_family: usize) -> Vec<TypeConstraints> {
        if rest.is_empty() { 
            let type_cutoff = interpretation.get_types().len() - signature.params.len();
            self.note_method(&signature, interpretation.get_types().clone(), aut, method_family);
            return vec![interpretation.clone().cut_to(type_cutoff)]; 
        }
        let mut types = vec![];
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                types.append(&mut self.infer_method_types_rec(&prod, &rest[1..], signature, aut, method_family));
            }
        }
        types
    }

    fn note_method(&self, signature: &Signature, types: Vec<VariableType>, aut: &mut Automaton, method_family: usize) {
        // // swap signature
        let mut new_signature = vec![];
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
