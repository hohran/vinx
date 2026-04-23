use std::fmt::Display;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Effect {
    Blur,
    Random,
    Inverse,
}

impl Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blur => write!(f, "blur"),
            Self::Random => write!(f, "random"),
            Self::Inverse => write!(f, "inverse"),
        }
    }
}
