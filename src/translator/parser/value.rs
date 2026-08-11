use image::Rgb;

use crate::{translator::{self, ast, error::CompilationError, parser::parser::Parser, word::Word}, variable::{Position, Variable, VariableType, VariableValue}};

pub enum ValueParseError {
    UnknownVariableName(String),
    HeterogenousVector,
}

pub type OperationId = usize;
pub type StructureId = usize;


impl Parser {
    pub fn parse_value(&self, val: &ast::Value) -> Result<VariableValue, ValueParseError> {
        match val {
            ast::Value::Number(n) => Ok(VariableValue::Int(*n as i32)),
            ast::Value::Variable(name) => {
                match self.globals.get_variable(name) {
                    Some(v) => Ok(v.clone()),
                    None => Err(ValueParseError::UnknownVariableName(name.clone()))
                }
            }
            ast::Value::Color(c) => Ok(VariableValue::Color(Rgb([c.0,c.1,c.2]))),
            ast::Value::Effect(e) => Ok(VariableValue::Effect(*e)),
            ast::Value::String(s) => Ok(VariableValue::String(s.clone())),
            ast::Value::Position(p) => Ok(VariableValue::Pos(Position::new(p.0 as i32, p.1 as i32))),
            ast::Value::Direction(d) => Ok(VariableValue::Direction(*d)),
            ast::Value::Vector(v) => {
                let mut out = vec![];
                let mut vec_type = VariableType::Any(0);
                for elem in v {
                    let elem_val = self.parse_value_as_variable(elem)?;
                    if let Some(new_vec_type) = vec_type.intersect(&elem_val.get_type()) {
                        vec_type = new_vec_type;
                    } else {
                        return Err(ValueParseError::HeterogenousVector);
                    }
                    out.push(elem_val);
                }
                Ok(VariableValue::Vec(out))
            }
        }
    }

    pub fn parse_value_as_variable(&self, val: &ast::Value) -> Result<Variable, ValueParseError> {
        match val {
            ast::Value::Variable(name) => {
                match self.globals.get_variable(name) {
                    Some(v) => Ok(Variable::Named(name.clone(), v.get_type())),
                    None => Err(ValueParseError::UnknownVariableName(name.clone()))
                }
            }
            _ => Ok(Variable::Static(self.parse_value(val)?)),
        }
    }

    pub fn parse_sequence(&self, seq: &ast::Sequence) -> Result<(translator::Sequence, Vec<Variable>), CompilationError> {
        let mut words = vec![];
        let mut params = vec![];
        for w in seq {
            match &w.0 {
                ast::sequence::Word::Keyword(k) => words.push(Word::Keyword(k.clone())),
                ast::sequence::Word::Value(v) => {
                    let var = match self.parse_value_as_variable(v) {
                        Ok(v) => v,
                        Err(value_error) => {
                            match value_error {
                                ValueParseError::UnknownVariableName(name) 
                                    => return Err(CompilationError::UnknownVariableName(name, self.get_location(&w.1))),
                                ValueParseError::HeterogenousVector 
                                    => return Err(CompilationError::TemporaryError("heterogenous array".to_string())), // TODO: change
                            }
                        }
                    };
                    words.push(Word::Type(var.get_type()));
                    params.push(var);
                }
            }
        }
        Ok((translator::Sequence::from(words), params))
    }

    pub fn parse_type(&self, typ: &ast::Type) -> Result<VariableType, CompilationError> {
        let mut t = match typ.value.as_ref() {
            "Int" => VariableType::Int,
            "Pos" => VariableType::Pos,
            "Column" => VariableType::Column,
            "Row" => VariableType::Row,
            "Color" => VariableType::Color,
            "String" => VariableType::String,
            "Effect" => VariableType::Effect,
            "Direction" => VariableType::Direction,
            "Rectangle" => VariableType::Rectangle,
            "Image" => VariableType::Image,
            x => panic!("error: unknown type {x}")
        };
        t.wrap_depth(typ.depth);
        Ok(t)
    }
}
