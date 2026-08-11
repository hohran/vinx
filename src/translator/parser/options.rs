/// Options are set from within the vinx program.
pub struct Options {
    pub save_video: bool,
}

impl Options {
    /// Create Options with default values.
    pub fn default() -> Self {
        Self { save_video: true }
    }
}
