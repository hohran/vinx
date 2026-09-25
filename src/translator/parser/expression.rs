use std::{fmt::Display, vec};

use image::Rgb;

use crate::{context, translator::{self, Sequence, SequenceValue, ast, error::{CompilationError, Location}, type_constraints::TypeConstraints, word::Word}, variable::{Position, Variable, VariableType, VariableValue}};
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

// expression interpretation
#[derive(Debug, Clone, Eq)]
pub struct ExprInt {
    pub expr_type: Option<VariableType>,
    pub constraint: TypeConstraints,
}

impl PartialEq for ExprInt {
    fn eq(&self, other: &Self) -> bool {
        match (&self.expr_type, &other.expr_type) {
            (Some(t1), Some(t2)) => t1.strictly_matches(t2) && self.constraint == other.constraint,
            (None, None) => self.constraint == other.constraint,
            _ => false
        }
    }
}

impl ExprInt {
    pub fn constant(t: VariableType) -> Self {
        Self { expr_type: Some(t), constraint: TypeConstraints::new() }
    }

    pub fn new(constraint: TypeConstraints, t: Option<VariableType>) -> Self {
        Self { expr_type: t, constraint }
    }

    pub fn get_with_type(self) -> (TypeConstraints, VariableType) {
        (self.constraint, self.expr_type.unwrap())
    }
}

impl Parser {
    /// Get expression interpretations: returns the list mapping types of ambiguous parameters (given by TypeConstraints) into the optional expression type.
    pub fn get_expression_interpretations(&mut self, expr: &ast::Expr) -> Vec<ExprInt> {
        match expr {
            ast::Expr::Value(term) => self.get_term_interpretations(term),
            ast::Expr::Call(call) => self.get_call_interpretations(call),
        }
    }

    fn get_term_interpretations(&mut self, term: &ast::Term) -> Vec<ExprInt> {
        match term {
            ast::Term::Direction(_) => vec![ExprInt::constant(VariableType::Direction)],
            ast::Term::Number(_) => vec![ExprInt::constant(VariableType::Int)],
            ast::Term::Color(_) => vec![ExprInt::constant(VariableType::Color)],
            ast::Term::Effect(_) => vec![ExprInt::constant(VariableType::Effect)],
            ast::Term::String(_) => vec![ExprInt::constant(VariableType::String)],
            ast::Term::Position((x, y)) => {
                let int = ExprInt::constant(VariableType::Int);
                let x = self.get_expression_interpretations_matching(x, &int);
                let y = self.get_expression_interpretations_matching(y, &int);
                let mut ints = vec![];
                for x_int in x {
                    let x_constraint = &x_int.constraint;
                    for y_int in y.clone() {
                        let y_constraint = y_int.constraint.clone();
                        let Some(constraint) = x_constraint.clone().intersect(y_constraint) else {
                            continue;
                        };
                        ints.push(ExprInt::new(constraint, Some(VariableType::Pos)));
                    }
                }
                ints
            }
            ast::Term::Expression(e) => self.get_expression_interpretations(e),
            ast::Term::Variable((name, _)) => {
                match self.context.get_variable(name) {
                    Some(val) => vec![ExprInt::constant(val.get_type())],
                    None => panic!("error: expected var `{name}` to be defined"), // TODO: friendlify
                }
            }
            ast::Term::Vector(exprs) => {
                if exprs.len() == 0 {
                    todo!("empty arrays");
                }
                // TODO: use the previous constraints to trim all future ones (with new function
                // consuming the current constraint)
                let expression_interpretations: Vec<Vec<ExprInt>> = exprs
                    .iter()
                    .map(|e| self.get_expression_interpretations(e))
                    .collect();

                let elem_id = self.new_unresolved_variable();
                let default_interpretation = (TypeConstraints::new(), VariableType::Any(elem_id));
                let interpretations = self.infer_vec_interpretations_rec(default_interpretation, &expression_interpretations);

                self.resolve_variables(1);
                return interpretations;
            }
        }
    }

    fn get_expression_interpretations_matching(&mut self, expr: &ast::Expr, int: &ExprInt) -> Vec<ExprInt> {
        match expr {
            ast::Expr::Value(term) => self.get_term_interpretations_matching(term, int),
            ast::Expr::Call(call) => self.get_call_interpretations_matching(call, int),
        }
    }

    /// Prerequisity: type of int is set (not None)
    fn get_constant_term_interpretations_matching(&mut self, int: &ExprInt, constant_expression_type: VariableType) -> Vec<ExprInt> {
        let (constr, t) = int.clone().get_with_type();
        if constant_expression_type.is_assignable_to(&t) {
            vec![ExprInt::new(constr, Some(constant_expression_type))]
        } else {
            vec![]
        }
    }

    fn get_term_interpretations_matching(&mut self, term: &ast::Term, int: &ExprInt) -> Vec<ExprInt> {
        if int.expr_type.is_none() {
            return vec![]
        }
        match term {
            ast::Term::Expression(e) => self.get_expression_interpretations_matching(e, int),
            ast::Term::Variable((name, _)) => {
                let (mut constr, t) = int.clone().get_with_type();
                let Some(val) = self.context.get_variable(name) else {
                    panic!("error: expected var `{name}` to be defined")
                };
                let val_type = val.get_type();
                // check if var is ambiguous => constrained
                if let Some(binding) = val_type.get_binding() {
                    if !constr.intersect_var(binding, &t) { // we are enforcing the type of this
                                                            // variable to be the expected type
                        return vec![]
                    }
                    let val_type = constr.at(binding);
                    vec![ExprInt::new(constr, Some(val_type))]
                } else {
                    let Some(type_prod) = val.get_type().intersect(&t) else {
                        return vec![]
                    };
                    vec![ExprInt::new(constr, Some(type_prod))]
                }
            }
            ast::Term::Vector(_exprs) => todo!(),
            ast::Term::Direction(_) => self.get_constant_term_interpretations_matching(int, VariableType::Direction),
            ast::Term::Number(_) => self.get_constant_term_interpretations_matching(int, VariableType::Int),
            ast::Term::Color(_) => self.get_constant_term_interpretations_matching(int, VariableType::Color),
            ast::Term::Effect(_) => self.get_constant_term_interpretations_matching(int, VariableType::Effect),
            ast::Term::String(_) => self.get_constant_term_interpretations_matching(int, VariableType::String),
            ast::Term::Position(_) => todo!("position interpretation: ensure that both expression evaluate to a number"),
        }
    }

    fn infer_vec_interpretations_rec(&self, current: (TypeConstraints, VariableType), rest: &[Vec<ExprInt>]) -> Vec<ExprInt> {
        let (cur_int, cur_ret) = current;
        if rest.is_empty() {
            return vec![ExprInt::new(cur_int, Some(cur_ret))];
        }
        let mut interpretations = vec![];
        for int in &rest[0] {
            let Some(ret) = &int.expr_type else { continue; };
            let new_ret = ret.intersect(&cur_ret);
            if new_ret.is_none() { continue; }
            let Some(prod) = cur_int.clone().intersect(int.constraint.clone()) else { continue; };
            // ufff, this is ugly... we need to pass (TC, Type) to contains, but (TC, Type?) to the
            // next call, so we reassign next
            let next = ExprInt::new(prod, new_ret);
            if interpretations.contains(&next) { continue; }
            // let (prod, new_ret) = next;
            // let next = (prod, new_ret.unwrap());
            interpretations.append(&mut self.infer_vec_interpretations_rec(next.get_with_type(), &rest[1..]));
        }
        interpretations
    }

    fn get_call_interpretations(&mut self, call: &ast::Sequence) -> Vec<ExprInt> {
        self.get_call_interpretations_rec(Sequence::new(Location::default()), TypeConstraints::new(), call)
    }

    fn get_call_interpretations_rec(&mut self, mut cur_seq: Sequence, cur_int: TypeConstraints, rest: &[(ast::sequence::Word, ast::Range)]) -> Vec<ExprInt> {
        if rest.is_empty() {
            match self.automaton.run(cur_seq.get()) {
                Some(s) => {
                    let types: Vec<VariableType> = cur_seq.get_types().iter().map(|&t| {let x = t.clone(); x}).collect();
                    let t = s.get_return_type(&types);
                    return vec![ExprInt::new(cur_int, t)]
                }
                None => return vec![]
            }
        }
        match &rest[0].0 {
            ast::sequence::Word::Keyword(word) => {
                cur_seq.push(Word::Keyword(word.clone()));
                return self.get_call_interpretations_rec(cur_seq, cur_int, &rest[1..])
            }
            ast::sequence::Word::Value(term) => {
                let tmp_id = self.new_unresolved_variable();
                let term_constraint = ExprInt::new(cur_int, Some(VariableType::Any(tmp_id)));
                let term_ints = self.get_term_interpretations_matching(term, &term_constraint);
                self.resolve_variables(1);
                let mut out = vec![];
                for int in term_ints {
                    let (next_int, param_type) = int.get_with_type();
                    let mut next_seq = cur_seq.clone();
                    next_seq.push(Word::Type(param_type));
                    out.append(&mut self.get_call_interpretations_rec(next_seq, next_int, &rest[1..]));
                }
                out
            }
        }
    }

    fn get_call_interpretations_matching(&self, call: &ast::Sequence, int: &ExprInt) -> Vec<ExprInt> {
        todo!()
    }

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
                    let Some(new_vec_type) = vec_type.intersect(&elem.get_type_unchecked()) else {
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
                    sequence.push(Word::Type(expr.get_type_unchecked()));
                    params.push(expr);
                }
            }
        }
        let sequence = self.get_sequence_value(&sequence)?;
        Ok(Call { value: sequence, params })
    }
}

impl Expression {
    pub fn get_type_unchecked(&self) -> VariableType {
        match self {
            Self::Constant(v) => v.get_type(),
            Self::Vector(_, t) => t.clone(),
            Self::Position(_, _) => VariableType::Pos,
            Self::Variable(var) => var.get_type(),
            Self::Call(c) => c.get_return_type().unwrap(), // TODO: think about if this is only
                                                           // called safely
        }
    }

    pub fn get_type(&self) -> Option<VariableType> {
        if let Self::Call(c) = self {
            c.get_return_type()
        } else {
            Some(self.get_type_unchecked())
        }
    }

    pub fn expect_type(&self, expected: VariableType) -> Result<(), CompilationError> {
        let t = self.get_type_unchecked();
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
        let types = self.params.iter().map(|p| p.get_type_unchecked()).collect::<Vec<VariableType>>();
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

#[cfg(test)]
mod tests {
    use crate::{translator::ast::{Expr, Term}, variable::Direction};

    use super::*;

    #[test]
    fn test_get_term_interpretations() {
        let mut parser = Parser::placeholder();
        let term = Term::Direction(Direction::Up);
        let ints = parser.get_term_interpretations(&term);
        assert_eq!(ints.len(), 1);
        let int = &ints[0];
        assert_eq!(int.expr_type, Some(VariableType::Direction));
        assert!(int.constraint.is_empty());
        //
        let term = Term::Number(420);
        let ints = parser.get_term_interpretations(&term);
        assert_eq!(ints.len(), 1);
        let int = &ints[0];
        assert_eq!(int.expr_type, Some(VariableType::Int));
        assert!(int.constraint.is_empty());
        //
        let term = Term::String("a".into());
        let ints = parser.get_term_interpretations(&term);
        assert_eq!(ints.len(), 1);
        let int = &ints[0];
        assert_eq!(int.expr_type, Some(VariableType::String));
        assert!(int.constraint.is_empty());
        //
        let term = Term::Color((0,0,0));
        let ints = parser.get_term_interpretations(&term);
        assert_eq!(ints.len(), 1);
        let int = &ints[0];
        assert_eq!(int.expr_type, Some(VariableType::Color));
        assert!(int.constraint.is_empty());
    }
}
