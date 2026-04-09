use std::collections::HashMap;

use crate::video::{Frame, Video};
use crate::variable::VariableValue;

pub type Globals = HashMap<String,VariableValue>;

pub struct Context {
    video: Option<Video>,
    frame_idx: usize,
    register: Option<VariableValue>,
}

impl Context {
    pub fn empty() -> Self {
        Self { video: None, register: None, frame_idx: 0, }
    }

    pub fn from(video: Video) -> Self {
        Self { video: Some(video), register: None, frame_idx: 0, }
    }

    pub fn get_width(&self) -> usize {
        let Some(video) = &self.video else {
            panic!("error: empty context")
        };
        video.width() as usize
    }

    pub fn get_height(&self) -> usize {
        let Some(video) = &self.video else {
            panic!("error: empty context")
        };
        video.height() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.video.is_none()
    }

    pub fn step(&mut self) {
        self.frame_idx += 1;
    }

    pub fn get_frame(&mut self) -> &mut Frame {
        let Some(video) = &mut self.video else {
            panic!("error: empty context")
        };
        video.get_frame_mut(self.frame_idx-1)
    }

    pub fn get_video_length(&mut self) -> usize {
        let Some(video) = &self.video else {
            panic!("error: empty context")
        };
        video.length()
    }

    pub fn get_video(&mut self) -> &mut Video {
        let Some(video) = &mut self.video else {
            panic!("error: empty context")
        };
        video
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
