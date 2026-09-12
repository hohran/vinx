use std::fmt::Display;

use image::Rgb;

use crate::{context, translator::{self, SequenceValue, ast, error::{CompilationError, Location}, word::Word}, variable::{Position, Variable, VariableType, VariableValue}};
use crate::context::Context;

use super::parser::Parser;

#[derive(Clone,PartialEq,Debug)]
pub struct Call {
    pub value: SequenceValue,
    pub params: Vec<Expression>,
}

#[derive(Clone,PartialEq,Debug)]
pub enum Expression {
    Constant(VariableValue),
    Variable(Variable),
    Vector(Vec<Expression>, VariableType),
    Position(Box<Expression>,Box<Expression>),
    Call(Call),
}

impl Parser {
    pub fn parse_expression(&self, val: &ast::Expr) -> Result<Expression, CompilationError> {
        match val {
            ast::Expr::Call(sequence) =>
                self.parse_call_expression(sequence).map(|call| Expression::Call(call)),
            ast::Expr::Value(term) => 
                self.parse_static_expression(term),
        }
    }

    fn parse_static_expression(&self, val: &ast::Term) -> Result<Expression, CompilationError> {
        match val {
            ast::Term::Expression(expr) => self.parse_expression(expr),
            ast::Term::Variable(name) => {
                let var_value = self.get_variable_value(name)?;
                Ok(Expression::Variable(Variable::Named(name.0.clone(), var_value.get_type())))
            }
            ast::Term::Number(n) => Ok(Expression::Constant(VariableValue::Int(*n as i32))),
            ast::Term::Color(c) => Ok(Expression::Constant(VariableValue::Color(Rgb([c.0,c.1,c.2])))),
            ast::Term::Effect(e) => Ok(Expression::Constant(VariableValue::Effect(*e))),
            ast::Term::String(s) => Ok(Expression::Constant(VariableValue::String(s.clone()))),
            ast::Term::Direction(d) => Ok(Expression::Constant(VariableValue::Direction(*d))),
            ast::Term::Position((x, y)) => {
                let x = self.parse_expression(x)?;
                x.expect_type(VariableType::Int);
                let y = self.parse_expression(y)?;
                y.expect_type(VariableType::Int);
                Ok(Expression::Position(Box::new(x), Box::new(y)))
            }
            ast::Term::Vector(values) => {
                let mut v: Vec<Expression> = Vec::new();
                let mut vec_type = VariableType::Any(0);
                for expr in values {
                    let elem = self.parse_expression(expr)?;
                    let Some(new_vec_type) = vec_type.intersect(&elem.get_type()) else {
                        return Err(CompilationError::HeterogenousVector(v[0].clone(), elem));
                    };
                    vec_type = new_vec_type;
                    v.push(elem);
                }
                Ok(Expression::Vector(v, vec_type))
            }
        }
    }

    pub fn parse_call_expression(&self, seq: &ast::Sequence) -> Result<Call, CompilationError> {
        let mut params = vec![];
        let mut sequence = translator::Sequence::new(self.get_location(&seq.into()));
        for w in seq {
            match &w.0 {
                ast::sequence::Word::Keyword(k) => sequence.push(Word::Keyword(k.clone())),
                ast::sequence::Word::Value(v) => {
                    let expr = self.parse_static_expression(v)?;
                    sequence.push(Word::Type(expr.get_type()));
                    params.push(expr);
                }
            }
        }
        let sequence = self.get_sequence_value(&sequence)?;
        Ok(Call { value: sequence, params })
    }
}

impl Expression {
    pub fn get_type(&self) -> VariableType {
        match self {
            Self::Constant(v) => v.get_type(),
            Self::Vector(_, t) => t.clone(),
            Self::Position(_, _) => VariableType::Pos,
            Self::Variable(var) => var.get_type(),
            Self::Call(c) => c.get_return_type().unwrap(), // TODO: think about if this is only
                                                           // called safely
        }
    }

    pub fn expect_type(&self, expected: VariableType) -> Result<(), CompilationError> {
        let t = self.get_type();
        if t != expected {
            Err(CompilationError::UnexpectedType(self.to_string(), t, expected, self.get_location()))
        } else {
            Ok(())
        }
    }

    pub fn get_location(&self) -> Location {
        Location::default() // TODO FIXME FIXME FIXME
    }

    // pub fn evaluate(&self, context: &dyn context::Context) -> Option<Variable> {
    //     match self {
    //         Self::Constant(v) => Some(v.to_var()),
    //         Self::Vector(values, _) => {
    //             let mut v: Vec<Variable> = vec![];
    //             for expr in values {
    //                 v.push(expr.evaluate(context).unwrap());
    //             }
    //             Some(Variable::Static(VariableValue::Vec(v)))
    //         }
    //         Self::Position(x, y) => {
    //             let x_var = x.evaluate(context).unwrap();
    //             let y_var = y.evaluate(context).unwrap();
    //             let x = context.get_value(&x_var);
    //             let y = context.get_value(&y_var);
    //             Some(Variable::Static(VariableValue::Pos(Position::new(x.into_int(), y.into_int()))))
    //         }
    //         Self::Variable(var) => Some(var.clone()),
    //         Self::Call(c) => {
    //             c.evaluate(context)
    //         }
    //     }
    // }

    pub fn evaluate_at_compiletime(&self, context: &mut context::Compiletime) -> Option<Variable> {
        match self {
            Self::Constant(v) => Some(v.to_var()),
            Self::Vector(values, _) => {
                let mut v: Vec<Variable> = vec![];
                for expr in values {
                    v.push(expr.evaluate_at_compiletime(context).unwrap());
                }
                Some(Variable::Static(VariableValue::Vec(v)))
            }
            Self::Position(x, y) => {
                let x_var = x.evaluate_at_compiletime(context).unwrap();
                let y_var = y.evaluate_at_compiletime(context).unwrap();
                let x = context.get_value(&x_var);
                let y = context.get_value(&y_var);
                Some(Variable::Static(VariableValue::Pos(Position::new(x.into_int(), y.into_int()))))
            }
            Self::Variable(var) => Some(var.clone()),
            Self::Call(c) => {
                c.evaluate_at_compiletime(context).map(|val| val.to_var())
            }
        }
    }

    pub fn evaluate_at_runtime(&self, context: &mut context::Runtime) -> Option<Variable> {
        match self {
            Self::Constant(v) => Some(v.to_var()),
            Self::Vector(values, _) => {
                let mut v: Vec<Variable> = vec![];
                for expr in values {
                    v.push(expr.evaluate_at_runtime(context).unwrap());
                }
                Some(Variable::Static(VariableValue::Vec(v)))
            }
            Self::Position(x, y) => {
                let x_var = x.evaluate_at_runtime(context).unwrap();
                let y_var = y.evaluate_at_runtime(context).unwrap();
                let x = context.get_value(&x_var);
                let y = context.get_value(&y_var);
                Some(Variable::Static(VariableValue::Pos(Position::new(x.into_int(), y.into_int()))))
            }
            Self::Variable(var) => Some(var.clone()),
            Self::Call(c) => {
                c.evaluate_at_runtime(context).map(|val| val.to_var())
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(v) => write!(f, "{v}"),
            Self::Vector(values, _) => {
                write!(f, "[")?;
                for (i,expr) in values.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{expr}")?;
                }
                write!(f, "]")
            }
            Self::Position(x, y) => write!(f, "({x},{y})"),
            Self::Variable(var) => write!(f, "{var}"),
            Self::Call(c) => write!(f, "{c}"),
        }
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.get_signature())
    }
}

impl Call {
    pub fn get_return_type(&self) -> Option<VariableType> {
        let types = self.params.iter().map(|p| p.get_type()).collect::<Vec<VariableType>>();
        self.value.get_return_type(&types)
    }

    // pub fn evaluate(&self, operations: &Operations, structures: &Vec<StructureTemplate>, context: &dyn context::Context) -> Option<Variable> {
    //     let params = self.params.iter().map(|expr| expr.evaluate(context).unwrap()).collect();
    //     self.value.evaluate(params, operations, structures, context)
    // }

    pub fn evaluate_at_compiletime(&self, context: &mut context::Compiletime) -> Option<VariableValue> {
        self.value.evaluate_at_compiletime(&self.params, context)
    }

    pub fn evaluate_at_runtime(&self, context: &mut context::Runtime) -> Option<VariableValue> {
        self.value.evaluate_at_runtime(&self.params, context)
    }
}
