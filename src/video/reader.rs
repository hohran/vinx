use super::*;
use ffmpeg_next::{format, frame, software::scaling};

// the lifetime is of a ffmpeg input
pub struct VideoReader<'a> {
    width: u32,
    height: u32,
    framerate: Rational,
    decoder: ffmpeg_next::decoder::Video,
    stream_index: usize,
    packets: ffmpeg_next::format::context::input::PacketIter<'a>,
    current_frame: ffmpeg_next::frame::Video,
    current_frame_index: usize,
    scaler: ffmpeg_next::software::scaling::Context,
    frame_count: usize,
    _eof_sent: bool,
}

impl<'a> VideoReader<'a> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn framerate(&self) -> Rational {
        self.framerate
    }

    pub fn new(input: &'a mut ffmpeg_next::format::context::Input) -> Self {
        let stream = input.streams().best(ffmpeg_next::media::Type::Video).unwrap();
        let frame_count = stream.frames() as usize;
        if frame_count == 0 {
            panic!("error: could not retrieve the frame count of video");
        }
        let stream_index = stream.index();
        let context_decoder = ffmpeg_next::codec::context::Context::from_parameters(stream.parameters()).unwrap();
        let decoder = context_decoder.decoder().video().unwrap();
        let width = decoder.width();
        let height = decoder.height();
        let packets = input.packets();
        let current_frame = frame::Video::empty();
        let framerate = decoder.frame_rate().expect("could not get the video frame rate");
        let scaler = scaling::Context::get(
            decoder.format(),
            width,
            height,
            format::Pixel::RGB24,
            width,
            height,
            scaling::Flags::BILINEAR,
        ).unwrap();
        Self { width, height, decoder, stream_index, packets, current_frame, scaler, framerate, _eof_sent: false, frame_count, current_frame_index: 0 }
    }

    fn transform_current_frame(&mut self) -> Frame {
        let mut rgb_frame = frame::Video::empty();
        self.scaler.run(&self.current_frame, &mut rgb_frame).unwrap();
        let mut buffer: Frame = image::ImageBuffer::new(rgb_frame.width(), rgb_frame.height());

        for (x, y, pixel) in buffer.enumerate_pixels_mut() {
            let data = rgb_frame.data(0);
            let stride = rgb_frame.stride(0) as usize;
            let offset = y as usize * stride + x as usize * 3;
            *pixel = image::Rgb([
                data[offset],
                data[offset + 1],
                data[offset + 2],
            ]);
        }
        buffer
    }

    pub fn get_frame_index(&self) -> usize {
        self.current_frame_index
    }

    pub fn get_next_frame(&mut self) -> Option<Frame> {
        if self.decoder.receive_frame(&mut self.current_frame).is_ok() {
            self.current_frame_index += 1;
            return Some(self.transform_current_frame())
        }
        while let Some((stream, packet)) = self.packets.next() {
            if stream.index() != self.stream_index { continue; }
            self.decoder.send_packet(&packet).unwrap();
            if self.decoder.receive_frame(&mut self.current_frame).is_ok() {
            self.current_frame_index += 1;
                return Some(self.transform_current_frame())
            }
        }
        if !self._eof_sent {
            self._eof_sent = true;
            self.decoder.send_eof().unwrap();
        }
        if self.decoder.receive_frame(&mut self.current_frame).is_ok() {
            self.current_frame_index += 1;
            return Some(self.transform_current_frame())
        }
        None
    }
}
