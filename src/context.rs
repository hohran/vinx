use crate::video::{Frame, VideoReader};

pub struct Context<'a> {
    video_reader: Option<&'a mut VideoReader<'a>>,
    current_frame: Option<Frame>,
}

impl<'a> Context<'a> {
    pub fn is_empty(&self) -> bool {
        self.current_frame.is_none()
    }

    pub fn new() -> Self {
        Self { video_reader: None, current_frame: None }
    }

    pub fn set_reader(&mut self, reader: &'a mut VideoReader<'a>) {
        self.video_reader = Some(reader);
    }

    pub fn get_frame_index(&self) -> usize {
        self.video_reader.as_ref().expect("error: no reader given").get_frame_index()
    }

    pub fn load_next_frame(&mut self) -> bool {
        let r = self.video_reader.as_mut().expect("error: could not find video reader");
        self.current_frame = r.get_next_frame();
        self.current_frame.is_some()
    }

    pub fn has_frame(&self, msg: &str) {
        println!("{} ... {}", msg, self.current_frame.is_some());
    }

    pub fn pop_current_frame(&mut self) -> Frame {
        self.current_frame.take().expect("error: no current frame loaded")
    }

    pub fn get_current_frame(&self) -> &Frame {
        self.current_frame.as_ref().expect("error: no current frame loaded")
    }

    pub fn get_current_frame_mut(&mut self) -> &mut Frame {
        self.current_frame.as_mut().expect("error: no current frame loaded")
    }

    pub fn get_width(&self) -> usize {
        let Some(video) = &self.video_reader else {
            panic!("error: empty context")
        };
        video.width() as usize
    }

    pub fn get_height(&self) -> usize {
        let Some(video) = &self.video_reader else {
            panic!("error: empty context")
        };
        video.height() as usize
    }
}

// pub struct ContextOld {
//     video: Option<Video>,
//     frame_idx: usize,
//     register: Option<VariableValue>,
// }
//
// impl ContextOld {
//     pub fn empty() -> Self {
//         Self { video: None, register: None, frame_idx: 0, }
//     }
//
//     pub fn from(video: Video) -> Self {
//         Self { video: Some(video), register: None, frame_idx: 0, }
//     }
//
//     pub fn get_width(&self) -> usize {
//         let Some(video) = &self.video else {
//             panic!("error: empty context")
//         };
//         video.width() as usize
//     }
//
//     pub fn get_height(&self) -> usize {
//         let Some(video) = &self.video else {
//             panic!("error: empty context")
//         };
//         video.height() as usize
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.video.is_none()
//     }
//
//     pub fn step(&mut self) {
//         self.frame_idx += 1;
//     }
//
//     pub fn get_frame(&mut self) -> &mut Frame {
//         let Some(video) = &mut self.video else {
//             panic!("error: empty context")
//         };
//         video.get_frame_mut(self.frame_idx-1)
//     }
//
//     pub fn get_video_length(&mut self) -> usize {
//         let Some(video) = &self.video else {
//             panic!("error: empty context")
//         };
//         video.length()
//     }
//
//     pub fn get_video(&mut self) -> &mut Video {
//         let Some(video) = &mut self.video else {
//             panic!("error: empty context")
//         };
//         video
//     }
//
//     pub fn set_register(&mut self, val: VariableValue) {
//         self.register = Some(val);
//     }
//
//     /// get the value of register and unset it
//     pub fn get_register(&mut self) -> VariableValue {
//         if let Some(v) = self.register.clone() {
//             self.register = None;
//             v
//         } else {
//             panic!("error: register is unset");
//         }
//     }
// }
