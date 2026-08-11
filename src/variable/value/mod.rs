mod structure;
mod direction;
mod effect;
mod rectangle;
mod position;
mod column;
mod value;

pub type Color = image::Rgb<u8>;

pub use structure::Structure;
pub use direction::Direction;
pub use effect::Effect;
pub use rectangle::Rectangle;
pub use position::Position;
pub use value::VariableValue;
pub use column::{Column,Row};
