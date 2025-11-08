extern crate tree_sitter;
extern crate tree_sitter_vinx;

mod automata;
mod type_inference;
mod translator;
pub use translator::parse;
mod builtins;
mod component_class;

use crate::{event::variable::types::VariableType, vtype};

#[derive(Clone,Hash,Eq,PartialEq,Debug)]
pub enum Word {
    Keyword(String),
    Type(VariableType),
    Label,
}

impl Word {
    pub fn is_type(&self) -> bool {
        matches!(self, Word::Type(_))
    }

    pub fn is_ambiguous(&self) -> bool {
        match self {
            Word::Type(vt) => vt.is_ambiguous(),
            _ => false,
        }
    }

    pub fn get_binding(&self) -> Option<usize> {
        match self {
            Word::Type(vt) => vt.get_binding(),
            _ => None,
        }
    }

    pub fn get_variable_type(&self) -> Option<VariableType> {
        match self {
            Word::Type(vt) => Some(vt.clone()),
            _ => None,
        }
    }
}

impl ToString for Word {
    fn to_string(&self) -> String {
        match self {
            Word::Label => { "@".to_string() }
            Word::Keyword(k) => { k.clone() }
            Word::Type(t) => { t.to_string() }
        }
    }
}

fn get_bounded_value(ambiguous_word: &Word, parsed_word: &Word) -> (usize,Word) {
    match ambiguous_word {
        Word::Type(VariableType::Any(binding)) => (*binding,parsed_word.clone()),
        Word::Type(VariableType::Vec(ambi)) => {
            if let Word::Type(VariableType::Vec(parsed)) = parsed_word {
                get_bounded_value(&Word::Type(*ambi.clone()), &Word::Type(*parsed.clone()))
            } else {
                panic!("error: expected parsed_word to match ambiguous_word, found {:?} and {:?}", parsed_word, ambiguous_word);
            }
        }
        _ => {
            panic!("error: expected ambiguous_word to be ambiguous, got {:?}", ambiguous_word)
        }
    }
}

pub type Sequence = Vec<Word>;
fn seq_to_str(seq: &Sequence) -> String {
    let mut ret = "[".to_string();
    for w in seq {
        ret.push(' ');
        ret.push_str(&w.to_string());
    }
    ret.push_str(" ]");
    ret
}

#[macro_export]
macro_rules! word {
    ( [ $($x:tt)+ ] ) => { Word::Type(VariableType::Vec(Box::new(vtype!($($x)+)))) };
    ( Int ) => { Word::Type(VariableType::Int) };
    ( Pos ) => { Word::Type(VariableType::Pos) };
    ( Color ) => { Word::Type(VariableType::Color) };
    ( Direction ) => { Word::Type(VariableType::Direction) };
    ( Component($i:expr) ) => { Word::Type(VariableType::Component($i)) };
    ( Any ( $i:expr ) ) => { Word::Type(VariableType::Any($i)) };
    ( ( $($x:tt)+ ) ) => { word!($($x)+) };
    ( Label ) => { Word::Label };
    ( $x:ident ) => { Word::Keyword(stringify!($x).to_string()) };
    ( $x:expr ) => { Word::Keyword($x.to_string()) };
}


#[derive(Copy,Clone,Eq,PartialEq,Debug)]
pub enum SequenceValue {
    Operation(usize),
    Component(usize),
}

#[macro_export]
macro_rules! seq {
    ( $($x:tt)+ ) => {
        ([$(word!($x)),+]).to_vec()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_word_macro() {
        assert_eq!(word!(Int),Word::Type(VariableType::Int));
        assert_eq!(word!(Pos),Word::Type(VariableType::Pos));
        assert_eq!(word!(Color),Word::Type(VariableType::Color));
        assert_eq!(word!(Direction),Word::Type(VariableType::Direction));
        assert_eq!(word!(Any(1)),Word::Type(VariableType::Any(1)));
        assert_eq!(word!([[Int]]),Word::Type(VariableType::Vec(Box::new(VariableType::Vec(Box::new(VariableType::Int))))));
        assert_eq!(word!(Label),Word::Label);
        assert_eq!(word!(ahoj),Word::Keyword("ahoj".to_string()));
        assert_eq!(word!("ahoj"),Word::Keyword("ahoj".to_string()));
    }

    #[test]
    fn test_seq_macro() {
        assert_eq!(seq!(rotate [Int] into Label "as" (Any(1))), vec![word!(rotate),word!([Int]),word!(into),word!(Label),word!("as"),word!(Any(1))]);
    }
}
