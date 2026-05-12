use crate::event::Operation;
use crate::translator::StructureTemplate;
use crate::translator::sequence::Sequence;
use crate::{seq, word, vtype};
use crate::variable::VariableType;
use crate::event::{builtins::*};

use super::{automata::Automaton, SequenceValue, Word};

macro_rules! builtin {
    (($($w:tt)+), $f:expr, $r:expr) => {
        ((seq!($($w)+)), Some($r), $f)
    };

    (($($w:tt)+), $f:expr) => {
        ((seq!($($w)+)), None, $f)
    };
}

macro_rules! builtins {
    (
        $(
            ($($w:tt)+) $(=> $r:expr)?, $f:expr
        );* $(;)?
    ) => {
        [
            $(
                builtin!(($($w)+), $f $(, $r)?)
            ),*
        ]
    };
}

/// Generate all builtin operations, note them in the automaton `aut` and return them.
pub fn load_builtin_operations(aut: &mut Automaton) -> Vec<Operation> {
    let builtins: &[(Sequence, Option<VariableType>, Builtin)] = &builtins!(
        ("restricted" "move" Pos Direction "by" Int), move_pos;
        ("move" Pos Direction "by" Int), move_pos_phase;
        ("draw" Color "rectangle" "outline" "from" Pos "to" Pos), draw_rect_outline;
        ("activate" String), activate;
        ("deactivate" String), deactivate;
        ("set" (Any(0)) "to" (Any(0))), set;
        ("rotate" [(Any(0))] Direction "by" Int), rotate_vec;
        ("top" [(Any(0))] "into" (Any(0))), top_into;
        ("add" Int "to" Int), add_to;
        ("draw" Color "rectangle" "from" Pos "to" Pos), draw_rect;
        ("draw" Effect "rectangle" "from" Pos "to" Pos), draw_effect_rect;
        ("toggle" String), toggle_activeness;
        ("sub" Int "from" Int), sub;
        ("move" Pos "by" Pos), move_by;
        ("print" String), print;
        ("get" "frame") => VariableType::Image, get_frame;
        ("move" Rectangle "by" Pos), rectangle::move_by;
        ("expand" Rectangle "by" Int), rectangle::expand;
        ("draw" Color Rectangle), rectangle::draw;
        ("draw" Image "at" Pos), image::draw_at;
        ("save" Image "as" String), image::save_as;
        ("draw" Color Rectangle "into" Image), image::draw_into;
        (Color "image" Int "x" Int) => VariableType::Image, image::colored;
        ("load" "image" "from" String) => VariableType::Image, image::load_from;
    );
    let mut ops = vec![];
    for (i,(seq,ret,op)) in builtins.into_iter().enumerate() {
        if !aut.register(seq.clone(), SequenceValue::Operation(i)) {
            panic!("error: union did not create any new states");
        }
        ops.push(Operation::from_builtin(i, seq.clone(), *op, ret.clone()));
    }
    ops
}

/// Generate all builtin structures, note them in the automaton `aut` and return them.
pub fn load_builtin_structures(aut: &mut Automaton) -> Vec<StructureTemplate> {
    let builtins = [
        // rectangle
        seq!("rectangle" "from" Pos "to" Pos),
    ];
    let mut structures = vec![];
    for (i,seq) in builtins.into_iter().enumerate() {
        let types = seq.get_types();
        if !aut.register(seq.clone(), SequenceValue::Structure(i)) {
            panic!("error: union did not create any new states");
        }
        let param_names: Vec<String> = types.iter().enumerate().map(|(i,_)| i.to_string()).collect();
        let param_types = types.into_iter().map(|v| v.clone()).collect();
        structures.push(StructureTemplate::new(i, param_names, param_types, vec![]));
    }
    structures
}
