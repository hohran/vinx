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
        seq!("set" (Any(0)) "to" (Any(0))),
        // rotate vector
        seq!("rotate" [(Any(0))] Direction "by" Int),
        // top into
        seq!("top" [(Any(0))] "into" (Any(0))),
        // add to
        seq!("add" Int "to" Int),
        // draw rect
        seq!("draw" Color "rectangle" "from" Pos "to" Pos),
        // draw effect rect
        seq!("draw" Effect "rectangle" "from" Pos "to" Pos),
        // toggle activity
        seq!("toggle" String),
        // sub
        seq!("sub" Int "from" Int),
        ];
    let mut la;
    let num_of_builtins = builtins.len();
    for (i,op) in builtins.into_iter().enumerate() {
        la = LinearAutomaton::new(op, SequenceValue::Operation(i+1));
        if !aut.union(la) {
            panic!("error: union did not create any new states");
        }
    }
    num_of_builtins
}
