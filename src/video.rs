extern crate rsframe;
pub use rsframe::vfx::video::{Frame,Video};
// extern crate image;
//
// #[derive(Debug,Clone)]
// pub struct Frame {
//     buffer: Vec<u8>,
//     width: usize,
//     height: usize,
// }
//
// impl Frame {
//     pub fn new(width: usize, height: usize) -> Self {
//         Self { buffer: vec![255;3*width*height], width, height }
//     }
//
//     pub fn save(&self, path: &str) {
//         let _ = image::save_buffer(path, &self.buffer, self.width as u32, self.height as u32, image::ColorType::Rgb8);
//     }
// }
