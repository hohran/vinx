use crate::event::{Func, OperationTemplate, OperationTemplateEnum, TopLevelOperation, builtins::*};
use crate::translator::{SequenceValue, Signature};
use crate::{seq, word, vtype};
use crate::variable::VariableType;

use super::{automata::Automaton, Word, StructureTemplate, Sequence};

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
pub fn load_top_level_operations(aut: &mut Automaton) -> Vec<OperationTemplateEnum> {
    let builtins: &[(Sequence, TopLevelOperation)] = &[
        (seq!("load" String), TopLevelOperation::LoadFile),
        (seq!("do" "not" "save"), TopLevelOperation::DoNotSave),
    ];
    let mut ops = vec![];
    for (seq,f) in builtins {
        let op = OperationTemplateEnum::TopLevel(*f);
        if !aut.register(seq.clone(), SequenceValue::Operation(op.clone())) {
            panic!("error: union did not create any new states");
        }
        ops.push(op);
    }
    ops
}

/// Generate all builtin operations, note them in the automaton `aut` and return them.
pub fn load_runtime_builtin_operations(aut: &mut Automaton, id_prefix: usize) -> Vec<OperationTemplateEnum> {
    let builtins: &[(Sequence, Option<VariableType>, BuiltinRuntime)] = &builtins!(
        ("draw" Color "rectangle" "outline" "from" Pos "to" Pos), draw_rect_outline;
        ("activate" String), activate;
        ("deactivate" String), deactivate;
        ("stop"), stop;
        ("draw" Color "rectangle" "from" Pos "to" Pos), draw_rect;
        ("draw" Effect "rectangle" "from" Pos "to" Pos), draw_effect_rect;
        ("toggle" String), toggle_activeness;
        ("get" "frame") => VariableType::Image, get_frame;
        ("draw" Color Rectangle), rectangle::draw;
        ("draw" Color "outline" "of" Rectangle), rectangle::draw_outline;
        ("draw" Image "at" Pos), image::draw_at;
        ("take" "column" "at" Int) => VariableType::Column, column::take;
        ("take" "row" "at" Int) => VariableType::Row, row::take;
    );
    let mut ops = vec![];
    for (i,(seq,ret,op)) in builtins.into_iter().enumerate() {
        let op = OperationTemplateEnum::Standard(OperationTemplate::from_builtin(id_prefix + i, seq.clone(), Func::RuntimeBuiltin(*op), ret.clone()));
        if !aut.register(seq.clone(), SequenceValue::Operation(op.clone())) {
            panic!("error: union did not create any new states");
        }
        // FIXME: id
        ops.push(op);
    }
    ops
}

/// Generate all builtin operations, note them in the automaton `aut` and return them.
pub fn load_builtin_operations(aut: &mut Automaton, id_prefix: usize) -> Vec<OperationTemplateEnum> {
    let builtins: &[(Sequence, Option<VariableType>, Builtin)] = &builtins!(
        ((Any(0))) => VariableType::Any(0), get_value;
        ("set" (Any(0)) "to" (Any(0))), set;
        ("rotate" [(Any(0))] Direction "by" Int), rotate_vec;
        ("top" [(Any(0))] "into" (Any(0))), top_into;
        ("top" [Any(0)]) => VariableType::Any(0), top;
        ("add" Int "to" Int), add_to;
        (Int "plus" Int) => VariableType::Int, plus;
        ("sub" Int "from" Int), sub;
        (Int "minus" Int) => VariableType::Int, minus;
        ("move" Pos "by" Pos), move_by;
        ("print" String), print;
        ("debug" (Any(0))), debug;
        ("move" Rectangle "by" Pos), rectangle::move_by;
        ("expand" Rectangle "by" Int), rectangle::expand;
        ("get" "corner" "of" Rectangle) => VariableType::Pos, rectangle::get_corner;
        ("save" Image "as" String), image::save_as;
        ("draw" Color Rectangle "into" Image), image::draw_into;
        ("rectangle" "from" Pos "to" Pos) => VariableType::Rectangle, rectangle::new;
        (Color "image" Int "x" Int) => VariableType::Image, image::colored;
        ("load" "image" "from" String) => VariableType::Image, image::load_from;
        ("take" Rectangle "from" Image) => VariableType::Image, image::take_from;
        ("append" Column "to" Image), column::append;
        ("prepend" Column "to" Image), column::prepend;
        ("append" Row "to" Image), row::append;
        ("prepend" Row "to" Image), row::prepend;
    );
    let mut ops = vec![];
    for (i,(seq,ret,op)) in builtins.into_iter().enumerate() {
        let op = OperationTemplateEnum::Standard(OperationTemplate::from_builtin(id_prefix + i, seq.clone(), Func::Builtin(*op), ret.clone()));
        if !aut.register(seq.clone(), SequenceValue::Operation(op.clone())) {
            panic!("error: union did not create any new states");
        }
        ops.push(op);
    }
    ops
}

/// Generate all builtin structures, note them in the automaton `aut` and return them.
pub fn load_builtin_structures(aut: &mut Automaton) -> Vec<StructureTemplate> {
    let builtins: &[Sequence] = &[
        // // rectangle
        // seq!("rectangle" "from" Pos "to" Pos),
    ];
    let mut structures = vec![];
    for (i,seq) in builtins.into_iter().enumerate() {
        let types = seq.get_types();
        let param_names: Vec<String> = types.iter().enumerate().map(|(i,_)| i.to_string()).collect();
        let param_types = types.into_iter().map(|v| v.clone()).collect();
        let s = StructureTemplate::new(i, param_names, param_types, vec![], Signature::from(seq.clone()));
        if !aut.register(seq.clone(), SequenceValue::Structure(s.clone())) {
            panic!("error: union did not create any new states");
        }
        structures.push(s);
    }
    structures
}
