mod reader;
mod writer;
mod image_processing;

pub type Frame = image::RgbImage;

use ffmpeg_next::Rational;
pub use writer::{Video, VideoWriter};
pub use reader::VideoReader;
pub use image_processing::{Drawable, Extendable};
pub use ffmpeg_next::format::input as get_input;
