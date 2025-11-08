
use crate::event::variable::types::VariableType;

use super::{automata::{automaton::Automaton, linear_automaton::LinearAutomaton}, SequenceValue, Word};

pub fn load_builtin_operations() -> (Automaton,usize) {
    let builtins = [
        // move pos
        vec![
            Word::Keyword("restricted".to_string()),
            Word::Keyword("move".to_string()),
            Word::Type(VariableType::Pos),
            Word::Type(VariableType::Direction),
            Word::Keyword("by".to_string()),
            Word::Type(VariableType::Int),
        ],
        // move pos phase
        vec![
            Word::Keyword("move".to_string()),
            Word::Type(VariableType::Pos),
            Word::Type(VariableType::Direction),
            Word::Keyword("by".to_string()),
            Word::Type(VariableType::Int),
        ],
        // draw rect
        vec![
            Word::Keyword("draw".to_string()),
            Word::Type(VariableType::Color),
            Word::Keyword("rectangle".to_string()),
            Word::Keyword("from".to_string()),
            Word::Type(VariableType::Pos),
            Word::Keyword("to".to_string()),
            Word::Type(VariableType::Pos),
        ],
        // activate
        vec![
            Word::Keyword("activate".to_string()),
            Word::Label,
        ],
        // deactivate
        vec![
            Word::Keyword("deactivate".to_string()),
            Word::Label,
        ],
        // set
        vec![
            Word::Keyword("set".to_string()),
            Word::Type(VariableType::Any(1)),
            Word::Keyword("to".to_string()),
            Word::Type(VariableType::Any(1)),
        ],
        // rotate vector
        vec![
            Word::Keyword("rotate".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Any(1)))),
            Word::Type(VariableType::Direction),
            Word::Keyword("by".to_string()),
            Word::Type(VariableType::Int),
        ],
        // top into
        vec![
            Word::Keyword("top".to_string()),
            Word::Type(VariableType::Vec(Box::new(VariableType::Any(1)))),
            Word::Keyword("into".to_string()),
            Word::Type(VariableType::Any(1)),
        ],
        ];
        let mut a = Automaton::new();
        let mut la;
        for (i,op) in builtins.iter().enumerate() {
            la = LinearAutomaton::from(op);
            la.returns(SequenceValue::Operation(i+1));
            if let Err(e) = a.union(la) {
                panic!("error: {e}");
            }
        }
        (a,builtins.len()+1)
}
