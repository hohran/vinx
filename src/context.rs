use std::collections::HashMap;

use crate::video::{Frame, Video};
use crate::variable::values::VariableValue;

pub type Globals = HashMap<String,VariableValue>;

pub struct Context {
    video: Video,
    frame_idx: usize,
    register: Option<VariableValue>,
}

impl Context {
    pub fn from(video: Video) -> Self {
        Self { video, register: None, frame_idx: 0, }
    }

    pub fn get_width(&self) -> usize {
        self.video.width
    }

    pub fn get_height(&self) -> usize {
        self.video.height
    }

    pub fn step(&mut self) {
        self.frame_idx += 1;
    }

    pub fn get_frame(&mut self) -> &mut Frame {
        self.video.get_frame_mut(self.frame_idx-1)
    }

    pub fn get_video_length(&mut self) -> usize {
        self.video.length()
    }

    pub fn get_video(&mut self) -> &mut Video {
        &mut self.video
    }

    pub fn set_register(&mut self, val: VariableValue) {
        self.register = Some(val);
    }

    /// get the value of register and unset it
    pub fn get_register(&mut self) -> VariableValue {
        if let Some(v) = self.register.clone() {
            self.register = None;
            v
        } else {
            panic!("error: register is unset");
        }
    }
}
