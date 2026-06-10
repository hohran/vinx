use crate::{translator::{Sequence, Signature, ast, error::CompilationError, parser::parser::Parser, word::Word}, variable::{VariableType, VariableValue}};

impl Parser {
    pub fn is_forbidden_variable_name(&self, name: &str) -> bool {
        name == self.self_reference_name
    }

    pub fn parse_var_definition(&mut self, var_definition: &ast::VarDefinition) -> Result<(), CompilationError> {
        let (seq,params) = self.get_sequence(&var_definition.value)?;
        let value = match self.automaton.run(seq.get()) {
            Some(sv) => sv.into_value(params, &self.operations, &self.structures, &mut self.globals),
            None => return Err(CompilationError::UnknownSequence(seq, self.placeholder_location()))
        };
        let name = &var_definition.name;
        if self.is_forbidden_variable_name(name) {
            return Err(CompilationError::ForbiddenVariableName(name.clone(), self.placeholder_location()));
        }
        if self.globals.add_variable(name.clone(), value.clone()) {
            Ok(())
        } else {
            Err(CompilationError::RedeclaredVariable(name.clone(), self.placeholder_location()))
        }
    }

    pub fn parse_definition(&mut self, definition: &ast::Definition) -> Result<(), CompilationError> {
        let structure_proof = definition.body.iter().find(|n| matches!(n, ast::definition::Statement::Definition(_)));
        let operation_proof = definition.body.iter().find(|n| matches!(n, ast::definition::Statement::Event(_)));
        if structure_proof.is_some() && operation_proof.is_some() {
            return Err(CompilationError::VagueDefinition(
                    self.placeholder_location(),
                    self.placeholder_location(),
                    self.placeholder_location()))
        }
        self.globals.push(); {
            if structure_proof.is_some() {
                self.parse_structure(definition)?;
            } else {
                self.parse_operation(definition)?;
            }
        } self.globals.pop();
        Ok(())
    }

    pub fn parse_signature(&mut self, signature: &ast::Signature) -> Result<Signature, CompilationError> {
        let mut sequence = Sequence::new();
        let mut params = vec![];
        let mut iterators = vec![];
        let mut structure_param_id = None;
        let mut has_main_iterator = false;
        for word in signature {
            match word {
                ast::signature::Word::Keyword(k) => sequence.push(Word::Keyword(k.clone())),
                ast::signature::Word::Variable(name) => {
                    let param_id = self.new_unresolved_variable();
                    sequence.push(Word::Type(VariableType::Any(param_id)));
                    params.push(name.clone());
                    if name == self.self_reference_name {
                        if structure_param_id.is_some() {
                            return Err(CompilationError::TemporaryError(format!("multiple self reference names in signature {signature:?}")));
                        }
                        structure_param_id = Some(params.len()-1);
                    }
                    self.globals.add_variable(name.clone(), VariableValue::Any(param_id));
                }
                ast::signature::Word::Iterator(i) => {
                    let (name, is_main) = i;
                    let param_id = self.new_unresolved_variable();
                    params.push(name.clone());
                    if name == self.self_reference_name {
                        if structure_param_id.is_some() {
                            return Err(CompilationError::TemporaryError(format!("multiple self reference names in signature {signature:?}")));
                        }
                        structure_param_id = Some(params.len()-1);
                    }
                    sequence.push(Word::Type(VariableType::Any(param_id)));
                    self.globals.add_variable(name.clone(), VariableValue::Any(param_id));
                    let var_id = self.globals.top().len()-1;
                    if *is_main {
                        if has_main_iterator {
                            return Err(CompilationError::MultipleMainIterators(self.placeholder_location()));
                        }
                        has_main_iterator = true;
                        iterators.insert(0, var_id);
                    } else {
                        iterators.push(var_id);
                    }
                }
            }
        }
        Ok(Signature { sequence, params, iterators, structure_param_id })
    }
}
