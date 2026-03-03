use std::collections::HashSet;

use tree_sitter::Node;

use crate::{translator::{SequenceValue, StructureTemplate, automata::{automaton::Automaton, linear_automaton::LinearAutomaton}, get_children, operations::OperationRepr, seq_to_str, translator::Kind, type_constraints::TypeConstraints}, variable::{stack::VariableMap, types::VariableType, values::VariableValue}, word};

use super::{translator::Translator, Sequence, Word};

impl Translator {
    pub fn parse_structure(&mut self, signature_node: &Node, definition_node: &Node) {
        let (signature, operands, its) = self.parse_lhs(signature_node);
        assert!(its.len() == 0, "error: iterators not allowed in structure definition");    // TODO: user friendlify
        self.handle_structure(definition_node, &operands, &signature);
    }

    pub fn push_signature_params_to_scope(&mut self, params: &Vec<String>) {
        for i in 0..params.len() {
            let name = params[i].to_string();
            let var_id = self.new_unresolved_variable();
            self.globals.add_variable(name, VariableValue::Any(var_id));
        }
    }

    fn parse_method_signature(&mut self, signature_node: &Node) -> OperationRepr {
        signature_node.expect_kind("signature", self);
        let structure_ref_name = "$self";
        let mut op_signature = vec![];
        let mut op_params = vec![];
        let mut has_main_iterator = false;
        let mut iterators = vec![];
        let mut structure_param_id = None;
        for word_node in get_children(signature_node) {
            match word_node.kind() {
                "keyword" => op_signature.push(Word::Keyword(self.text(&word_node).to_string())),
                "variable" => {
                    let param_id = self.new_unresolved_variable();
                    let param_name = self.get_variable_name(&word_node).unwrap();  // variable always has a valid name
                    op_signature.push(word!(Any(param_id)));
                    op_params.push(param_name.to_string());
                    if param_name == structure_ref_name {
                        assert!(structure_param_id.is_none());  // TODO: friendlify
                        structure_param_id = Some(param_id);
                    }
                    self.globals.add_variable(param_name.to_string(), VariableValue::Any(param_id));
                }
                "iterator" => {
                    let var_node = word_node.child_by_field_name("variable").expect("error: iterator without variable field");
                    var_node.expect_kind("variable", self);
                    let param_id = self.new_unresolved_variable();
                    let param_name = self.get_variable_name(&var_node).unwrap();  // variable always has a valid name
                    op_params.push(param_name.to_string());
                    if param_name == structure_ref_name {
                        assert!(structure_param_id.is_none());  // TODO: friendlify
                        structure_param_id = Some(param_id);
                    }
                    op_signature.push(Word::Type(VariableType::Any(param_id)));
                    self.globals.add_variable(param_name.to_string(), VariableValue::Any(param_id));
                    let var_id = self.globals.top().len()-1;
                    if word_node.child_by_field_name("main").is_some() {    // main iterator
                        assert!(has_main_iterator == false);    // TODO: nice error message / warning
                        has_main_iterator = true;
                        iterators.insert(0, var_id);
                    } else {
                        iterators.push(var_id);
                    }
                }
                x => panic!("error: unexpected type {x} in sequence")
            }
        }
        assert!(structure_param_id.is_some());   // TODO: friendlify
        OperationRepr::new(op_signature, op_params, iterators, structure_param_id)
    }

    fn method_member_assert(&self, members: &Vec<String>, operation: &OperationRepr, structure_params: &Vec<String>) -> bool {
        for n in members {
            assert!(!structure_params.contains(&n));  // TODO: user friendlify
            assert!(!operation.params.contains(&n));  // TODO: user friendlify
            assert_ne!(n, "&self");  // TODO: user friendlify
        }
        true
    }

    fn handle_structure(&mut self, definition_node: &Node, operands: &Vec<String>, signature: &Vec<Word>) {
        self.globals.push_layer();
        self.push_signature_params_to_scope(operands);
        let mut method_aut = Automaton::new();
        let (structure_interpretations,methods) = self.get_structure_interpretations(operands, definition_node, &mut method_aut);
        for int in structure_interpretations {
            let structure_id = self._number_of_builtin_structures + self.structures.len();
            self.update_variables(operands, int.get_types());
            self.create_typed_structure(signature, operands, int.get_types()[..operands.len()].to_vec(), definition_node, structure_id);
            self.create_methods(&method_aut, &methods, structure_id, &get_children(definition_node));
        }
        self.globals.pop_layer();
    }

    fn get_structure_interpretations(&mut self, operands: &Vec<String>, definition_node: &Node, method_aut: &mut Automaton) -> (HashSet<TypeConstraints>,Vec<(OperationRepr,usize)>) {
        let mut methods: Vec<(OperationRepr,usize)> = vec![];
        let mut structure_interpretations = HashSet::new();
        let mut member_names = vec![];
        structure_interpretations.insert(TypeConstraints::_new());
        let stmts = get_children(definition_node);
        for i in 0..stmts.len() {
            let stmt = &stmts[i];
            match stmt.kind() {
                "var_definition" => {
                    let var_id = self.new_unresolved_variable();
                    let name = self.get_var_definition_name(stmt).to_string();
                    member_names.push(name.to_string());
                    self.globals.add_variable(name.clone(), VariableValue::Any(var_id));
                    let seq = self.get_sequence(&stmt.child_by_field_name("rhs").unwrap());
                    structure_interpretations = self.update_structure_interpretations_with_var(structure_interpretations, var_id, &seq);
                }
                "definition" => {
                    self.globals.push_layer();
                    let op_nodes = get_children(&stmt);
                    let op = self.parse_method_signature(&op_nodes[0]);
                    let (op_members,interpretations) = self.parse_operation_definition(&op_nodes[1], &op.signature, Some(method_aut));
                    assert!(self.method_member_assert(&op_members, &op, operands)); // TODO friendlify
                    if interpretations.len() == 0 { println!("nothing for structure"); return (HashSet::new(),vec![]); }   // TODO: warning
                    let constraint_size = operands.len()+member_names.len()+op.params.len();
                    structure_interpretations = self.update_structure_interpretations(structure_interpretations, &op, interpretations, constraint_size, method_aut, i);
                    self.resolve_variables(op.params.len()+op_members.len());
                    methods.push((op,i));
                    self.globals.pop_layer();
                }
                x => {
                    panic!("error: structure definition can only contain variable and operation definitions, got {x:?}")
                }
            }
        }
        self.resolve_variables(operands.len()+member_names.len());
        (structure_interpretations,methods)
    }

    fn update_variables(&mut self, names: &Vec<String>, types: &Vec<VariableType>) {
        for i in 0..names.len() {
            self.globals.update_variable(&names[i], types[i].default());
        }
    }

    fn create_methods(&mut self, method_aut: &Automaton, methods: &Vec<(OperationRepr,usize)>, structure_id: usize, stmts: &Vec<Node<'_>>) {
        for (method_signature, sv) in method_aut.get_all_sequences() {
            let SequenceValue::Operation(method_node_id) = sv else { panic!(); };
            let (op,_) = methods.iter().find(|m| m.1 == method_node_id).expect("error: could not find specified node");
            self.globals.push_layer();
            let (new_signature,str_id) = self.rewrite_method_signature(structure_id, method_signature, op);
            let mut events = vec![];
            let mut op_members = vec![];
            let method_node = &stmts[method_node_id];
            for method_stmt in get_children(&get_children(method_node)[1]) {
                match method_stmt.kind() {
                    "var_definition" => {
                        let name = self.get_var_definition_name(&method_stmt);
                        let (seq,params) = self.get_sequence_with_params(&method_stmt.child_by_field_name("rhs").unwrap());
                        let sv = self.action_decision_automaton.run(&seq).expect("error: unknown sequence");
                        op_members.push((name.to_string(),sv.clone(),params));
                    }
                    "sequence" => {
                        let mut event = self.get_event(&method_stmt);
                        if let Some(op) = self.operations.get(&event.get_id()) {
                            if let Some(struct_id) = op.method_of() {
                                if *struct_id == structure_id {
                                    event.deactivate_struct();
                                }
                            }
                        }
                        events.push(event);
                    }
                    x => panic!("error: unexpected int op definition: {x}"),
                }
            }
            self.globals.pop_layer();
            self.add_operation(new_signature, op.params.clone(), events, &op.iterators, op_members, str_id);
        }
    }

    fn rewrite_method_signature(&mut self, structure_id: usize, method_signature: Vec<Word>, operation: &OperationRepr) -> (Sequence,Option<usize>) {
        let mut new_signature = vec![];
        let mut str_id = None;
        let mut pushed_vars = 0;
        for w in method_signature {
            let Word::Type(t) = w.clone() else { 
                new_signature.push(w);
                continue; 
            };
            let param_name = operation.params[pushed_vars].clone();
            if param_name == "$self" {
                let str_t = VariableType::Component(structure_id);
                self.globals.add_variable(param_name, str_t.default());
                new_signature.push(Word::Type(str_t));
                str_id = Some(pushed_vars);
            } else {
                new_signature.push(w);
                self.globals.add_variable(param_name, t.default());
            }
            pushed_vars += 1;
        }
        (new_signature,str_id)
    }

    fn update_structure_interpretations(&mut self, structure_interpretations: HashSet<TypeConstraints>, method: &OperationRepr, method_interpretations: Vec<Vec<TypeConstraints>>, constraint_size: usize, aut: &mut Automaton, method_family: usize) -> HashSet<TypeConstraints> {
        let mut new_structure_ints = HashSet::new();
        for mut int in structure_interpretations {
            int.resize_to(constraint_size);
            let new_ints = self.infer_method_types_rec(&int, &method_interpretations, &method.signature, &method.params, &method.iterators, aut, method_family);
            for i in new_ints {
                new_structure_ints.insert(i);
            }
        }
        new_structure_ints
    }

    fn update_structure_interpretations_with_var(&mut self, structure_interpretations: HashSet<TypeConstraints>, var_id: usize, rhs: &Sequence) -> HashSet<TypeConstraints> {
        let mut new_structure_ints = HashSet::new();
        for mut struct_int in structure_interpretations {
            assert_eq!(struct_int.get_types().len()+2, var_id);
            if rhs.len() == 1 && let Some(t) = rhs[0].get_variable_type() {
                let ret = struct_int.intersect_var(&t, var_id);
                assert!(ret);
                new_structure_ints.insert(struct_int);
                continue;
            }
            for var_int in self.action_decision_automaton.get_interpretations(rhs, Some(var_id)) {
                assert_eq!(var_int.get_types().len()+1, var_id);
                if let Some(new_int) = struct_int.clone().intersect(var_int) {
                    new_structure_ints.insert(new_int);
                }
            }
        }
        new_structure_ints
    }

    /// insert new structure
    fn create_typed_structure(&mut self, signature: &Vec<Word>, operands: &Vec<String>, types: Vec<VariableType>, node: &Node, id: usize) {
        // swap signature
        let mut new_signature = vec![];
        for w in signature {
            if let Word::Type(VariableType::Any(var_id)) = w {
                let v = &types[*var_id];
                new_signature.push(Word::Type(v.clone()));
                self.globals.update_variable(&operands[*var_id], v.default());
            } else {
                new_signature.push(w.clone());
            }
        }
        let mut members = vec![];
        for stmt in get_children(node) {
            if stmt.kind() != "var_definition" {
                continue;
            }
            let name = self.get_var_definition_name(&stmt).to_string();
            let (seq,params) = self.get_sequence_with_params(&stmt.child_by_field_name("rhs").unwrap());
            let sv = self.action_decision_automaton.run(&seq).expect("error: unknown sequence");
            match sv {
                SequenceValue::Operation(_) => {
                    todo!("operation returns");
                }
                SequenceValue::Component(id) => {
                    self.globals.update_variable(&name, VariableType::Component(id).default());
                }
                SequenceValue::Value(ref t) => {
                    self.globals.update_variable(&name, t.default());
                }
            }
            members.push((name,sv,params));
        }
        // println!("adding structure {id} with signature \"{}\"", seq_to_str(&new_signature));
        let la = LinearAutomaton::new(new_signature, SequenceValue::Component(id));
        if !self.action_decision_automaton.union(la) {
            return;
        }
        let structure = StructureTemplate::new(id, operands.clone(), types, members);
        self.structures.push(structure);
    }

    fn infer_method_types_rec(&mut self, interpretation: &TypeConstraints, rest: &[Vec<TypeConstraints>], method_signature: &Vec<Word>, method_params: &Vec<String>, iterators: &Vec<usize>, aut: &mut Automaton, method_family: usize) -> Vec<TypeConstraints> {
        if rest.is_empty() { 
            let type_cutoff = interpretation.get_types().len() - method_params.len();
            self.note_method(method_signature, interpretation.get_types().clone(), iterators, aut, method_family);
            return vec![interpretation.clone().cut_to(type_cutoff)]; 
        }
        let mut types = vec![];
        for int in &rest[0] {
            if let Some(prod) = interpretation.clone().intersect(int.clone()) {
                types.append(&mut self.infer_method_types_rec(&prod, &rest[1..], method_signature, method_params, iterators, aut, method_family));
            }
        }
        types
    }

    fn is_builtin_structure(&self, id: usize) -> bool {
        id <= self._number_of_builtin_structures
    }

    fn note_method(&self, signature: &Sequence, types: Vec<VariableType>, iterators: &Vec<usize>, aut: &mut Automaton, method_family: usize) {
        // swap signature
        let mut new_signature = vec![];
        for w in signature {
            if let Word::Type(VariableType::Any(var_id)) = w {
                let v = if types.len() > *var_id { &types[*var_id] } else { &VariableType::Any(*var_id) };
                if iterators.contains(var_id) {
                    new_signature.push(Word::Type(VariableType::Vec(Box::new(v.clone()))));
                } else {
                    new_signature.push(Word::Type(v.clone()));
                }
            } else {
                new_signature.push(w.clone());
            }
        }
        let op_id = method_family;
        let la = LinearAutomaton::new(new_signature, SequenceValue::Operation(op_id));
        aut.union(la);
    }
}
