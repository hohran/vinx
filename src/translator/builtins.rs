use crate::{seq, word, vtype};
use crate::variable::types::VariableType;

use super::{automata::{automaton::Automaton, linear_automaton::LinearAutomaton}, SequenceValue, Word};

pub fn load_builtin_operations(aut: &mut Automaton) -> usize {
    let builtins = [
        // move pos
        seq!("restricted" "move" Pos Direction "by" Int),
        // move pos phase
        seq!("move" Pos Direction "by" Int),
        // draw rect outline
        seq!("draw" Color "rectangle" "outline" "from" Pos "to" Pos),
        // activate
        seq!("activate" String),
        // deactivate
        seq!("deactivate" String),
        // set
        seq!("set" (Any(1)) "to" (Any(1))),
        // rotate vector
        seq!("rotate" [(Any(1))] Direction "by" Int),
        // top into
        seq!("top" [(Any(1))] "into" (Any(1))),
        // add to
        seq!("add" Int "to" Int),
        // draw rect
        seq!("draw" Color "rectangle" "from" Pos "to" Pos),
        // draw effect rect
        seq!("draw" Effect "rectangle" "from" Pos "to" Pos),
        // toggle activity
        seq!("toggle" String),
        ];
        let mut la;
        for (i,op) in builtins.iter().enumerate() {
            la = LinearAutomaton::from(op);
            la.returns(SequenceValue::Operation(i+1));
            if let Err(e) = aut.union(la) {
                panic!("error: {e}");
            }
        }
        builtins.len()+1
}

// pub fn load_builtin_values(aut: &mut Automaton) {
//     let builtins = [
//         seq!((Any(1))),
//         seq!(column at Int),
//     ];
//     let mut la;
//     for (i,seq) in builtins.iter().enumerate() {
//         la = LinearAutomaton::from(seq);
//         la.returns(SequenceValue::Value(i));
//         if let Err(e) = aut.union(la) {
//             panic!("error: {e}");
//         }
//     }
// }
