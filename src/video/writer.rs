use super::*;
use rayon::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use image::{ImageReader, ImageResult};
use std::{path::Path, process::Command};
use std::fs;
use rand::{Rng, distributions::Alphanumeric, thread_rng};

use ffmpeg_next::{codec, format, format::Pixel, software::scaling::{context::Context as ScalingContext, flag::Flags}, log};
use ffmpeg_next::util::frame::video::Video as VideoFrame;

pub struct VideoWriterOptions {
    pub codec: Vec<codec::id::Id>, // list of preffered codecs
    pub pixel_format: Pixel,
    pub bit_rate: usize,
    pub gop: u32,
    pub log_level: log::Level,
}

impl VideoWriterOptions {
    pub fn new(codec: Vec<codec::id::Id>, encoder_format: Pixel, bit_rate: usize, gop: u32, log_level: log::Level) -> Self {
        Self { codec, pixel_format: encoder_format, bit_rate, gop, log_level }
    }

    pub fn default(fps: u32) -> Self {
        Self { 
            codec: vec![codec::Id::H264, codec::Id::H265],
            pixel_format: Pixel::YUV420P,
            bit_rate: 4_000_000, 
            gop: fps,
            log_level: log::Level::Quiet,
        }
    }
}

pub struct VideoWriter {
    width: u32,
    height: u32,
    framerate: Rational,
    frames_added: i64,
    frames: Vec<ffmpeg_next::frame::Video>,
    config: VideoWriterOptions,
}

impl<'a> From<&VideoReader<'a>> for VideoWriter {
    fn from(value: &VideoReader) -> Self {
        let framerate = value.framerate();
        Self { 
            width: value.width(), height: value.height(), 
            framerate, frames_added: 0, frames: vec![], 
            config: VideoWriterOptions::default(framerate.into()) 
        }
    }
}

impl VideoWriter {
    fn configure_encoder(&self, encoder: &mut ffmpeg_next::codec::encoder::video::Video) {
        let fps = self.framerate;
        encoder.set_width(self.width);
        encoder.set_height(self.height);
        encoder.set_format(self.config.pixel_format);
        encoder.set_time_base(fps.invert());
        encoder.set_frame_rate(Some(fps));
        encoder.set_bit_rate(self.config.bit_rate);
        encoder.set_gop(self.config.gop);
    }

    fn get_codec(&self, conf: &VideoWriterOptions) -> Option<ffmpeg_next::codec::codec::Codec> {
        for codec_id in &conf.codec {
            if let Some(codec) = ffmpeg_next::encoder::find(*codec_id) {
                return Some(codec);
            }
        }
        None
    }

    pub fn save(&self, output_path: &str) -> Result<(), ffmpeg_next::Error> {
        let mut octx = format::output(&output_path)?;

        ffmpeg_next::log::set_level(self.config.log_level);

        let codec = self.get_codec(&self.config).ok_or(ffmpeg_next::Error::EncoderNotFound)?;

        let mut encoder = codec::context::Context::new_with_codec(codec)
            .encoder()
            .video()?;

        self.configure_encoder(&mut encoder);

        // Some containers (mp4) require "global header" flag
        if octx.format().flags()
            .contains(format::flag::Flags::GLOBAL_HEADER)
        {
            encoder.set_flags(codec::Flags::GLOBAL_HEADER);
        }

        let mut ost = octx.add_stream(codec)?;
        let stream_index = ost.index();

        let opened_encoder = match encoder.open_as(codec) {
            Ok(opened_encoder) => opened_encoder,
            Err(e) => panic!("error: could not open encoder: {e}")
        };
        ost.set_parameters(&opened_encoder);
        let mut encoder = opened_encoder;

        // Write out
        octx.write_header()?;
        for frame in &self.frames {
            encoder.send_frame(frame)?;
            let mut packet = ffmpeg_next::Packet::empty();
            while encoder.receive_packet(&mut packet).is_ok() {
                packet.set_stream(stream_index);
                packet.rescale_ts(encoder.time_base(), octx.stream(stream_index).unwrap().time_base());
                packet.write_interleaved(&mut octx)?;
            }
        }
        octx.write_trailer()?;
        Ok(())
    }

    pub fn append_frame(&mut self, img: Frame) -> Result<(), ffmpeg_next::Error> {
        let width = img.width();
        let height = img.height();

        // Frames are firstly converted to RGB24 pixel format with which ffmpeg can work
        // FIXME: it would be way more efficient to convert straight from `image` format
        let mut rgb_frame = VideoFrame::new(Pixel::RGB24, width, height);
        {
            let stride = rgb_frame.stride(0);
            let data = rgb_frame.data_mut(0);
            let raw = img.as_raw(); // tightly packed RGB24, no padding

            for y in 0..height as usize {
                let src_start = y * (width as usize) * 3;
                let src_end = src_start + (width as usize) * 3;
                let dst_start = y * stride;
                let dst_end = dst_start + (width as usize) * 3;
                data[dst_start..dst_end].copy_from_slice(&raw[src_start..src_end]);
            }
        }

        // Convert to whatever specified output format
        let mut scaler = ScalingContext::get(
            Pixel::RGB24,
            width,
            height,
            self.config.pixel_format,
            width,
            height,
            Flags::BILINEAR,
        )?;
        let mut frame = VideoFrame::empty();
        scaler.run(&rgb_frame, &mut frame)?;
        frame.set_pts(Some(self.frames_added));
        self.frames_added += 1;
        self.frames.push(frame);
        Ok(())
    }
}

pub struct Video {
    width: u32,
    height: u32,
    framerate: Rational,
    frames: Vec<Frame>,
}

impl<'a> From<VideoReader<'a>> for Video {
    fn from(value: VideoReader) -> Self {
        Self { width: value.width(), height: value.height(), frames: vec![], framerate: value.framerate() }
    }
}

impl Video {
    pub fn add_frame(&mut self, frame: &Frame) {
        self.frames.push(frame.clone());
    }

    pub fn save(&self, export_location: String, fps: u8, keep_folder: bool, ffmpeg: &str) {
        let temporary = create_tmp_folder();

        // Use Rayon to parallelize the loop
        self.frames.par_iter().enumerate().for_each(|(fi, frame)| {
            frame.save(format!("{}/image{}.bmp", temporary, fi + 1)).unwrap();
        });

        let results = build_folder(temporary.clone(), fps as i32, export_location, ffmpeg);

        match results {
            Ok(_) => {
                if !keep_folder {
                    drop_folder(temporary);
                }
            }
            Err(_) => {
                println!("Cannot render video.");
            }
        }
    }

    pub fn from_file(filename: String, ffmpeg: &str) -> Result<Video, String> {
        let temp = create_tmp_folder();

        eprintln!("Encoding video \"{filename}\"...");
        let encoding = ".bmp";
        let output = Command::new(ffmpeg)
            .arg("-i")
            .arg(filename.as_str())
            .arg("-loglevel").arg("quiet")
            .arg(format!("{temp}/image%d{encoding}"))
            .status();

        if let Err(err) = output {
            return Err(format!("FFmpeg command failed: {}", err));
        }

        if !output.unwrap().success() {
            return Err("FFmpeg did not exit successfully.".to_string());
        }

        let dir_path = Path::new(&temp);
        let entries: Vec<_> = match fs::read_dir(dir_path) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.is_file())
                .collect(),
            Err(err) => {
                drop_folder(temp);
                return Err(format!("Failed to read temporary directory: {}", err));
            }
        };

        let mut sorted_entries = entries.clone();
        sorted_entries.sort_by(|a, b| {
            // Extract frame numbers from filenames and compare numerically
            let a_num = a.file_name().unwrap().to_str().unwrap()
                .trim_start_matches("image")
                .trim_end_matches(encoding)
                .parse::<u32>().unwrap();
            let b_num = b.file_name().unwrap().to_str().unwrap()
                .trim_start_matches("image")
                .trim_end_matches(encoding)
                .parse::<u32>().unwrap();
            a_num.cmp(&b_num)
        });

        let frames: Vec<_> = sorted_entries
            .par_iter()
            .filter_map(|path| {
                let frame_path = path.to_str().unwrap().to_string();
                read_image(frame_path).ok()
            })
        .collect();

        if frames.is_empty() {
            drop_folder(temp);
            return Err("No frames were successfully loaded.".to_string());
        }

        let first_frame = &frames[0];
        let video = Video {
            width: first_frame.width(),
            height: first_frame.height(),
            frames,
            framerate: (24, 1).into(),
        };

        drop_folder(temp);

        Ok(video)
    }

    pub fn get_frame_mut(&mut self, at: usize) -> &mut Frame {
        &mut self.frames[at]
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn length(&self) -> usize {
        self.frames.len()
    }
}

pub fn build_folder(folder_path: String, framerate: i32, location: String, ffmpeg: &str) -> Result<(), ()> {
    // Ensure the input images exist
    let folder_path = Path::new(&folder_path);

    // Check for existing image files
    let image_files: Vec<_> = std::fs::read_dir(folder_path)
        .map_err(|_| ())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(ext) = entry.path().extension() {
                ext == "bmp"
            } else {
                false
            }
        })
        .collect();

    if image_files.is_empty() {
        eprintln!("No BMP images found in the specified folder");
        return Err(());
    }

    // Construct the input pattern for FFmpeg (all BMP files in the folder)
    let input_pattern = folder_path.join("image%d.bmp").to_string_lossy().to_string();

    // Execute FFmpeg command to convert images to video
    let output = Command::new(ffmpeg)
        .args(&[
            "-framerate", &framerate.to_string(),
            "-i", &input_pattern,
            "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2", // Ensure even resolution
            "-c:v", "libx264",  // Use H.264 video codec
            "-preset", "medium",
            "-crf", "23",        // Reasonable quality setting
            "-pix_fmt", "yuv420p", // Ensure compatibility
            "-y",  // Overwrite output file if it exists
            &location
        ])
        .output()
        .map_err(|_| ())?;  // Convert any execution error to ()

    // Check if the command was successful
    if output.status.success() {
        Ok(())
    } else {
        // Log the full error output
        eprintln!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr));
        Err(())
    }
}

pub fn read_image(path: String) -> ImageResult<Frame> {
    ImageReader::open(path)?.decode().map(|x| x.into())
}

fn drop_folder(path: String) {
    fs::remove_dir_all(path).expect("Could not drop folder.");
}

fn create_tmp_folder() -> String {
    let name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    let path = format!("_{}-tmp", name);

    std::fs::create_dir(path.clone()).unwrap();
    path
}
