use crate::{translator::{Sequence, Signature, ast::{self, Range}, error::{CompilationError, Location}, parser::{Expression, parser::Parser}, word::Word}, variable::{VariableType, VariableValue}};
use crate::context::Context;

pub struct VarDefinition {
    name: (String, Range),
    t: (VariableType, Range),
    value: Expression,
    is_type_only: bool,
}

impl VarDefinition {
    pub fn new(name: (String, Range), t: (VariableType, Range), value: Expression, is_type_only: bool) -> Self {
        Self { name, t, value, is_type_only }
    }

    pub fn get_name(&self) -> &String {
        &self.name.0
    }

    pub fn get_type(&self) -> &VariableType {
        &self.t.0
    }

    pub fn get_value(&self) -> &Expression {
        &self.value
    }

    pub fn get_value_location(&self) -> Location {
        self.value.get_location()
    }

    pub fn get_name_range(&self) -> &Range {
        &self.name.1
    }

    pub fn get_type_range(&self) -> &Range {
        &self.t.1
    }

    pub fn is_type_only(&self) -> bool {
        self.is_type_only
    }
}

impl Parser {
    pub fn is_forbidden_variable_name(&self, name: &str) -> bool {
        name == self.self_reference_name
    }

    /// var_id marks the binding, if the definition happens in a operation/structure definition.
    pub fn get_var_definition(&self, var_def: &ast::VarDefinition, var_id: Option<usize>) -> Result<VarDefinition, CompilationError> {
        let name = var_def.name.clone();
        match (&var_def.value, &var_def.typ) {
            (Some((val,_)), Some((t,tr))) => {
                let expr = self.parse_expression(val)?;
                let t = (self.parse_type(t)?, *tr);
                if !expr.get_type_unchecked().is_assignable_to(&t.0) {
                    return Err(CompilationError::TemporaryError(format!("type `{}` is not assignable to {}", expr.get_type_unchecked(), t.0)))
                }
                Ok(VarDefinition::new(name, t, expr, false))
            }
            (Some((val,_)), None) => {
                let expr = self.parse_expression(val)?;
                // let t = if let Some(binding) = var_id {
                //     VariableType::Any(binding)
                // } else {
                //     let sv = self.get_sequence_value(&seq)?;
                //     sv.get_general_return_type()
                // };
                // TODO: previously, we got a general return type, now a 'concrete'... think about it
                let t = var_id.map_or(expr.get_type_unchecked(), |binding| VariableType::Any(binding));
                Ok(VarDefinition::new(name, (t, Range::default()), expr, false))
            }
            (None, Some((t,tr))) => {
                let t = self.parse_type(t)?;
                let expr = Expression::Constant(t.default());
                Ok(VarDefinition::new(name, (t,*tr), expr, true))
            }
            _ => panic!("error: variable definition without type and value")
        }
    }

    // pub fn define_variable(&mut self, var_definition: &ast::VarDefinition) -> Result<(), CompilationError> {
    //     let var_definition = self.get_var_definition(var_definition, None)?;
    //     let (seq, params) = var_definition.get_value();
    //     let sv = self.get_sequence_value(seq)?;
    //     let value = sv.into_value(params.clone(), &self.operations, &self.structures, &mut self.context); // FIXME so that we dont clone params
    //     let name = var_definition.get_name();
    //     if self.is_forbidden_variable_name(name) {
    //         return Err(CompilationError::ForbiddenVariableName(name.clone(), var_definition.get_value_location()));
    //     }
    //     if self.context.add_variable(name.clone(), value.clone()) {
    //         Ok(())
    //     } else {
    //         Err(CompilationError::RedeclaredVariable(name.clone(), var_definition.get_value_location()))
    //     }
    // }

    // Define a variable in the given context, expecting its name is unique and not forbidden.
    pub fn define_variable(&mut self, var_definition: &ast::VarDefinition) -> Result<(), CompilationError> {
        let var_definition = self.get_var_definition(var_definition, None)?;
        let name = var_definition.get_name();
        if self.is_forbidden_variable_name(name) {
            return Err(CompilationError::ForbiddenVariableName(name.clone(), var_definition.get_value_location()));
        }
        let expr = var_definition.get_value();
        let Some(expr_var) = expr.evaluate_at_compiletime(&mut self.context) else {
            panic!("error: no return value");
        };
        let value = self.context.get_value(&expr_var).clone();
        if self.context.add_variable(name.clone(), value) {
            Ok(())
        } else {
            Err(CompilationError::RedeclaredVariable(name.clone(), var_definition.get_value_location()))
        }
    }

    pub fn parse_assignment(&mut self, assignment: &ast::Assignment) -> Result<(), CompilationError> {
        let name = &assignment.name.0;
        if self.context.get_stack().get_variable(name).is_none() {
            return Err(CompilationError::AssignmentOfUndefinedVariable(name.clone(), self.get_location(&assignment.name.1)))
        }
        let expr = self.parse_expression(&assignment.value.0)?;
        let Some(expr_var) = expr.evaluate_at_compiletime(&mut self.context) else {
            panic!("error: no return value");
        };
        let value = self.context.get_value(&expr_var).clone();
        let old_value = self.get_variable_value(&assignment.name)?;
        if !value.is_assignable_to(old_value) {
            return Err(CompilationError::UnexpectedType(name.clone(), value.get_type(), old_value.get_type(), self.get_location(&Range::from(assignment))));
        }
        self.context.update_variable(name, value);
        Ok(())
    }

    pub fn parse_definition(&mut self, definition: &ast::Definition) -> Result<(), CompilationError> {
        let structure_proof = definition.body.iter().find(|n| matches!(n, ast::definition::Statement::Definition(_)));
        let operation_proof = definition.body.iter().find(|n| matches!(n, ast::definition::Statement::Event(_)));
        if structure_proof.is_some() && operation_proof.is_some() {
            return Err(CompilationError::VagueDefinition(
                    self.get_location(&Range::from(&definition.signature)), // signature
                    self.get_location(&operation_proof.unwrap().into()), // seq
                    self.get_location(&structure_proof.unwrap().into()))) // method
        }
        self.context.push_scope(); {
            if structure_proof.is_some() {
                self.parse_structure(definition)?;
            } else {
                self.parse_operation(definition)?;
            }
        } self.context.pop_scope();
        Ok(())
    }

    pub fn parse_signature(&mut self, signature: &ast::Signature) -> Result<Signature, CompilationError> {
        let mut sequence = Sequence::new(self.get_location(&signature.into()));
        let mut params = vec![];
        let mut iterators = vec![];
        let mut structure_param_id = None;
        let mut has_main_iterator = false;
        for word in signature {
            match &word.0 {
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
                    self.context.add_variable(name.clone(), VariableValue::Any(param_id));
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
                    self.context.add_variable(name.clone(), VariableValue::Any(param_id));
                    let var_id = self.context.get_stack().top().len()-1;
                    if *is_main {
                        if has_main_iterator {
                            return Err(CompilationError::MultipleMainIterators(self.get_location(&word.1)));
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
