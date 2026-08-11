use std::fmt::Display;

use crate::variable::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rectangle {
    pub top_left: Position,
    pub bot_right: Position,
}

impl Rectangle {
    pub fn new(top_left: Position, bot_right: Position) -> Self {
        Self { top_left, bot_right }
    }

    pub fn default() -> Self {
        Self::new(Position::default(), Position::default())
    }
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rectangle from {} to {}", self.top_left, self.bot_right)
    }
}
