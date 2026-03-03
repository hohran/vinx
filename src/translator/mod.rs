extern crate tree_sitter;
extern crate tree_sitter_vinx;

mod automata;
mod type_constraints;
mod translator;
pub use translator::parse;
use tree_sitter::Node;
mod builtins;
// mod component_class;
mod operations;
mod structures;
mod value;
mod actions;
mod file_manager;

use crate::variable::{Variable, stack::{Stack, VariableMap}, types::VariableType, values::{Structure, VariableValue}};
// use crate::vtype;

#[derive(Debug)]
pub struct StructureTemplate {
    id: usize,
    param_names: Vec<String>,
    param_types: Vec<VariableType>,
    members: Vec<(String, SequenceValue, Vec<Variable>)>,
    // member_names: Vec<String>,
    // member_types: Vec<VariableType>,
    // - do it when sequences are allowed as assignments
}

impl StructureTemplate {
    pub fn new(id: usize, param_names: Vec<String>, param_types: Vec<VariableType>, members: Vec<(String, SequenceValue, Vec<Variable>)>) -> Self {
        Self { id, param_names, param_types, members }
    }

    pub fn instantiate(&self, params: Vec<Variable>, structures: &Vec<StructureTemplate>, stack: &mut Stack) -> Structure {
        assert_eq!(params.len(), self.param_names.len());
        // println!("instantiating structure {} with {params:?}", self.id);
        stack.push_layer();
        let mut members = VariableMap::new();
        for i in 0..params.len() {
            assert!(params[i].get_type() == &self.param_types[i]);
            members.insert(self.param_names[i].clone(), params[i].get_value(stack));
            stack.add_variable(self.param_names[i].clone(), params[i].get_value(stack));
        }
        for (name,val,ps) in &self.members {
            let member_val = match val {
                SequenceValue::Operation(_) => {
                    todo!("operation return values");
                }
                SequenceValue::Component(id) => {
                    let val = structures[*id].instantiate(ps.clone(), structures, stack);
                    VariableValue::Structure(val)
                }
                SequenceValue::Value(_) => {
                    assert_eq!(ps.len(), 1, "only 1 param for value");
                    ps[0].get_value(stack)
                }
            };
            members.insert(name.clone(), member_val.clone());
            stack.add_variable(name.clone(), member_val);
        }
        stack.pop_layer();
        let s = Structure::new(self.id, members);
        s
    }
}

/// Gets node children without comments
fn get_children<'a>(node: &Node<'a>) -> Vec<Node<'a>> {
    node.named_children(&mut node.walk()).filter(|n| n.kind() != "comment").collect()
}

/// Gets node children with unnamed symbols without comments
fn get_all_children<'a>(node: &Node<'a>) -> Vec<Node<'a>> {
    node.children(&mut node.walk()).filter(|n| n.kind() != "comment").collect()
}

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
            Word::Keyword(k) => { k.clone() }
            Word::Type(t) => { t.to_string() }
        }
    }
}

pub type Sequence = Vec<Word>;
pub fn seq_to_str(seq: &Sequence) -> String {
    let mut ret = "".to_string();
    for w in seq {
        ret.push_str(&w.to_string());
        ret.push(' ');
    }
    ret.pop();
    ret
}

#[macro_export]
macro_rules! word {
    ( [ $($x:tt)+ ] ) => { Word::Type(VariableType::Vec(Box::new(vtype!($($x)+)))) };
    ( Int ) => { Word::Type(VariableType::Int) };
    ( Pos ) => { Word::Type(VariableType::Pos) };
    ( Color ) => { Word::Type(VariableType::Color) };
    ( Direction ) => { Word::Type(VariableType::Direction) };
    ( Effect ) => { Word::Type(VariableType::Effect) };
    ( Component($i:expr) ) => { Word::Type(VariableType::Component($i)) };
    ( Any ( $i:expr ) ) => { Word::Type(VariableType::Any($i)) };
    ( ( $($x:tt)+ ) ) => { word!($($x)+) };
    ( String ) => { Word::Type(VariableType::String) };
    ( $x:ident ) => { Word::Keyword(stringify!($x).to_string()) };
    ( $x:expr ) => { Word::Keyword($x.to_string()) };
}

#[derive(Clone,Eq,PartialEq,Debug)]
pub enum SequenceValue {
    Operation(usize),
    Component(usize),
    Value(VariableType),
    // Value(usize),  // TODO: this would be nicer to remove: operations will have a return value and it will
            // be computed that way. potential problem is handling calling of operations in
            // build time: could they draw?
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
    use crate::vtype;
    
    #[test]
    fn test_word_macro() {
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

    #[test]
    fn test_word_eq() {
        let w1 = word!(Effect);
        let w2 = word!(Effect);
        assert_eq!(w1,w2);
    }

    #[test]
    fn test_seq_macro() {
        assert_eq!(seq!(rotate [Int] into "as" (Any(1)) String), vec![word!(rotate),word!([Int]),word!(into),word!("as"),word!(Any(1)),word!(String)]);
    }
}
