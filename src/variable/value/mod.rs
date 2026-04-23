mod structure;
mod direction;
mod effect;
mod value;

pub type Color = image::Rgb<u8>;

pub use structure::Structure;
pub use direction::Direction;
pub use effect::Effect;
pub use value::VariableValue;
