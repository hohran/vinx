use core::panic;

use image::Rgb;
use crate::context::Context;

use crate::{translator::{self, ast, error::CompilationError, parser::parser::Parser, word::Word}, variable::{Position, Variable, VariableType, VariableValue}};

/// Error kind for parsing values.
pub enum ValueParseError {
    UnknownVariableName(String),
    HeterogenousVector,
    UnexpectedType(String, VariableType, VariableType),
}

pub type OperationId = usize;
pub type StructureId = usize;

impl Parser {
    // fn get_variable_value_by_name(&self, var_name: &str) -> Result<&VariableValue, ValueParseError> {
    //     match self.context.get_variable(var_name) {
    //         Some(value) => Ok(value),
    //         None => Err(ValueParseError::UnknownVariableName(var_name.to_string())),
    //     }
    // }

    // fn parse_position_value(&self, val: &ast::PositionValue) -> Result<i32, ValueParseError> {
    //     match val {
    //         ast::PositionValue::Concrete(v) => Ok(*v as i32),
    //         ast::PositionValue::Variable(name) => {
    //             let val = self.get_variable_value_by_name(name)?;
    //             let VariableValue::Int(i) = val else {
    //                 return Err(ValueParseError::UnexpectedType(name.clone(), val.get_type(), VariableType::Int))
    //             };
    //             Ok(*i)
    //         }
    //     }
    // }

    // pub fn parse_term(&self, val: &ast::Term) -> Result<VariableValue, ValueParseError> {
    //     match val {
    //         ast::Term::Number(n) => Ok(VariableValue::Int(*n as i32)),
    //         ast::Term::Variable(name) => {
    //             let val = self.get_variable_value_by_name(&name.0)?;
    //             Ok(val.clone())
    //         }
    //         ast::Term::Color(c) => Ok(VariableValue::Color(Rgb([c.0,c.1,c.2]))),
    //         ast::Term::Effect(e) => Ok(VariableValue::Effect(*e)),
    //         ast::Term::String(s) => Ok(VariableValue::String(s.clone())),
    //         ast::Term::Position(p) => {
    //             let x = self.parse_position_value(&p.0)?;
    //             let y = self.parse_position_value(&p.1)?;
    //             Ok(VariableValue::Pos(Position::new(x, y)))
    //         }
    //         ast::Term::Direction(d) => Ok(VariableValue::Direction(*d)),
    //         ast::Term::Vector(v) => {
    //             let mut out = vec![];
    //             let mut vec_type = VariableType::Any(0);
    //             for elem in v {
    //                 let elem_val = self.parse_value_as_variable(elem)?;
    //                 let Some(new_vec_type) = vec_type.intersect(&elem_val.get_type()) else {
    //                     return Err(ValueParseError::HeterogenousVector);
    //                 };
    //                 vec_type = new_vec_type;
    //                 out.push(elem_val);
    //             }
    //             Ok(VariableValue::Vec(out))
    //         }
    //         ast::Term::Expression(expr) => {
    //             match self.parse_expression(expr)?.evaluate_at_compiletime(&mut self.context) {
    //                 Some(v) => Ok(self.context.get_value(&v)),
    //                 None => panic!(), // TODO: friendlify
    //             }
    //         }
    //     }
    // }

    // pub fn parse_value_as_variable(&self, val: &ast::Term) -> Result<Variable, ValueParseError> {
    //     match val {
    //         ast::Term::Variable(name) => {
    //             let val = self.get_variable_value_by_name(&name.0)?;
    //             Ok(Variable::Named(name.0.clone(), val.get_type()))
    //         }
    //         _ => Ok(Variable::Static(self.parse_term(val)?)),
    //     }
    // }

    // pub fn parse_sequence(&self, seq: &ast::Sequence) -> Result<(translator::Sequence, Vec<Variable>), CompilationError> {
    //     let mut params = vec![];
    //     let mut out = translator::Sequence::new(self.get_location(&seq.into()));
    //     for w in seq {
    //         match &w.0 {
    //             ast::sequence::Word::Keyword(k) => out.push(Word::Keyword(k.clone())),
    //             ast::sequence::Word::Value(v) => {
    //                 let var = match self.parse_value_as_variable(v) {
    //                     Ok(v) => v,
    //                     Err(value_error) => {
    //                         match value_error {
    //                             ValueParseError::UnknownVariableName(name) 
    //                                 => return Err(CompilationError::UnknownVariableName(name, self.get_location(&w.1))),
    //                             ValueParseError::HeterogenousVector 
    //                                 => return Err(CompilationError::TemporaryError("heterogenous array".to_string())), // TODO: change
    //                             ValueParseError::UnexpectedType(name, got, expected)
    //                                 => return Err(CompilationError::UnexpectedType(name, got, expected, self.get_location(&w.1))),
    //                         }
    //                     }
    //                 };
    //                 out.push(Word::Type(var.get_type()));
    //                 params.push(var);
    //             }
    //         }
    //     }
    //     Ok((out, params))
    // }

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
