use std::fmt::Display;

use crate::variable::VariableType;

/// Word represents a single word in signatures of functions.
/// For example `move Pos by Pos` contains 4 words, where `Type` is denoted with capital letters.
/// Words can be conveniently generated with the `word!` macro.
#[derive(Clone,Hash,Eq,PartialEq,Debug)]
pub enum Word {
    Keyword(String),
    Type(VariableType),
}

impl Word {
    pub fn is_type(&self) -> bool {
        matches!(self, Word::Type(_))
    }

    pub fn strictly_matches(&self, other: &Self) -> bool {
        match (self, other) {
            (Word::Keyword(s1),Word::Keyword(s2)) => s1 == s2,
            (Word::Type(t1),Word::Type(t2)) => t1.strictly_matches(t2),
            _ => false,
        }
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

    pub fn get_type(&self) -> Option<&VariableType> {
        match &self {
            Word::Type(vt) => Some(vt),
            _ => None
        }
    }
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Word::Keyword(k) => { write!(f, "{k}") }
            Word::Type(t) => { write!(f, "{t}") }
        }
    }
}

/// Generate words based on the convention:
///  * keywords are lowercase or wrapped with double quotes (necessery for keywords, such as move)
///  * types are corresponding to options in `VariableType`, case-sensitive
#[macro_export]
macro_rules! word {
    ( [ $($x:tt)+ ] ) => { Word::Type(VariableType::Vec(Box::new(vtype!($($x)+)))) };
    ( Int ) => { Word::Type(VariableType::Int) };
    ( Pos ) => { Word::Type(VariableType::Pos) };
    ( Color ) => { Word::Type(VariableType::Color) };
    ( Direction ) => { Word::Type(VariableType::Direction) };
    ( Effect ) => { Word::Type(VariableType::Effect) };
    ( Image ) => { Word::Type(VariableType::Image) };
    ( Structure ( $i:expr ) ) => { Word::Type(VariableType::Structure($i)) };
    ( Any ( $i:expr ) ) => { Word::Type(VariableType::Any($i)) };
    ( Rectangle ) => { Word::Type(VariableType::Structure(0)) };
    ( ( $($x:tt)+ ) ) => { word!($($x)+) };
    ( String ) => { Word::Type(VariableType::String) };
    ( $x:ident ) => { Word::Keyword(stringify!($x).to_string()) };
    ( $x:expr ) => { Word::Keyword($x.to_string()) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vtype;
    
    #[test]
    fn test_macro() {
        assert_eq!(word!(Int),Word::Type(VariableType::Int));
        assert_eq!(word!(Pos),Word::Type(VariableType::Pos));
        assert_eq!(word!(Color),Word::Type(VariableType::Color));
        assert_eq!(word!(Direction),Word::Type(VariableType::Direction));
        assert_eq!(word!(Any(1)),Word::Type(VariableType::Any(1)));
        assert_eq!(word!([[Int]]),Word::Type(VariableType::Vec(Box::new(VariableType::Vec(Box::new(VariableType::Int))))));
        assert_eq!(word!(String),Word::Type(VariableType::String));
        assert_eq!(word!(ahoj),Word::Keyword("ahoj".to_string()));
        assert_eq!(word!("ahoj"),Word::Keyword("ahoj".to_string()));
    }
}
