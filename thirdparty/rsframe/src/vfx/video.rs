use crate::vfx::{compile::build_folder, tmp::{drop_folder, create_tmp_folder}, text::create_text};
use image::{Rgb, RgbImage};
use std::process::Command;
use rayon::prelude::*;
use std::fs;
use std::path::Path;

/// A single pixel, typically used for representing a colour.
/// Pixels do not support alpha layers, so you should use `chroma_key` functions instead.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Pixel {
    pub r: u8, // Red component of the pixel
    pub g: u8, // Green component of the pixel
    pub b: u8, // Blue component of the pixel
}

impl Pixel {
    /// Creates a new pixel with the colours.
    pub fn new(r: u8, g: u8, b: u8) -> Pixel {
        Pixel {
            r,
            g,
            b
        }
    }

    /// Creates a black pixel
    pub fn black() -> Pixel {
        Pixel::new(0, 0, 0)
    }

    /// Creates a white pixel
    pub fn white() -> Pixel {
        Pixel::new(255, 255, 255)
    }

    /// Creates a red pixel
    pub fn red() -> Pixel {
        Pixel::new(255, 0, 0)
    }

    /// Creates a green pixel
    pub fn green() -> Pixel {
        Pixel::new(0, 255, 0)
    }

    /// Creates a blue pixel
    pub fn blue() -> Pixel {
        Pixel::new(0, 0, 255)
    }
}

/// The VideoPosition enum is used for determining where to put a transition effect in the video.
/// Currently, the enum is only used for the fade in transition.
pub enum VideoPosition {
    END,   // Indicates the transition effect should be applied at the end of the video
    START, // Indicates the transition effect should be applied at the start of the video
}

/// Private function; used for interpolating colours in the `tint` function.
/// 
/// # Arguments
/// 
/// * `t` - The interpolation factor, typically between 0 and 1.
/// * `start` - The starting value for interpolation.
/// * `end` - The ending value for interpolation.
/// 
/// # Returns
/// 
/// The interpolated value between `start` and `end`.
fn lerp(t: f32, start: f32, end: f32) -> f32 {
    start + (end - start) * t
}

/// A frame is a single frame in a video, it can be represented as an RGB image.
#[derive(Clone)]
pub struct Frame {
    pixels: Vec<Pixel>, // The pixel data of the frame
    pub width: usize,   // The width of the frame
    pub height: usize   // The height of the frame
}

impl Frame {
    /// Creates and returns a new frame with the background colour of your choice.
    /// 
    /// # Arguments
    /// 
    /// * `width` - The width of the frame.
    /// * `height` - The height of the frame.
    /// * `background` - The background colour of the frame.
    /// 
    /// # Returns
    /// 
    /// A new `Frame` instance.
    pub fn new(width: usize, height: usize, background: Pixel) -> Frame {
        let mut pixels = Vec::new();

        for _x in 0..width {
            for _y in 0..height {
                pixels.push(background.clone());
            }
        }

        Frame {
            width,
            height,
            pixels
        }
    }

    /// Returns a Frame of the text given. Requires ImageMagick installed and added to PATH.
    /// 
    /// # Arguments
    /// 
    /// * `width` - The width of the frame.
    /// * `height` - The height of the frame.
    /// * `font` - The font to be used for the text.
    /// * `color` - The colour of the text.
    /// * `text` - The text to be rendered.
    /// 
    /// # Returns
    /// 
    /// A new `Frame` instance containing the rendered text.
    pub fn text(width: usize, height: usize, font: String, color: String, text: String, magick: &str) -> Result<Frame, String> {
        let path = create_text(text, font, color, width, height, magick);

        let frame = Frame::from_img(path.clone());

        let rm_results = fs::remove_file(path);
        match rm_results {
            Ok(_) => {},
            Err(_) => eprintln!("Could not delete text image.")
        }

        Ok(frame?)
    }

    /// Returns a Frame that is an identical copy of the image provided.
    /// 
    /// # Arguments
    /// 
    /// * `image_path` - The path to the image file.
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the new `Frame` or an error message.
    pub fn from_img(image_path: String) -> Result<Frame, String> {
        let img = match image::open(image_path) {
            Ok(img) => img.to_rgb8(),
            Err(err) => return Err(format!("Failed to open image: {}", err)),
        };
        let mut pixels = Vec::with_capacity((img.width() * img.height()) as usize);

        for y in 0..img.height() {
            for x in 0..img.width() {
                let pixel_value = img.get_pixel(x, y).0;
                pixels.push(Pixel {
                    r: pixel_value[0],
                    g: pixel_value[1],
                    b: pixel_value[2],
                });
            }
        }

        Ok(Frame {
            width: img.width() as usize,
            height: img.height() as usize,
            pixels,
        })
    }

    /// Replaces the pixel at the given coordinates with the given colour.
    /// 
    /// # Arguments
    /// 
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// * `pixel` - The new pixel colour to set.
    pub fn put_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.pixels[y*self.width+x] = pixel;
    }

    /// Returns the pixel at the given coordinates, but not a mutable reference.
    /// 
    /// # Arguments
    /// 
    /// * `x` - The x-coordinate of the pixel.
    /// * `y` - The y-coordinate of the pixel.
    /// 
    /// # Returns
    /// 
    /// The pixel at the specified coordinates.
    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        self.pixels[y*self.width+x]
    }

    /// Replaces the current frame with a new image. The new image is the resolution given with a background colour of the colour given. The image is put in the centre of the new image.
    /// 
    /// # Arguments
    /// 
    /// * `target_width` - The width of the new frame.
    /// * `target_height` - The height of the new frame.
    /// * `background` - The background colour of the new frame.
    pub fn expand(&mut self, target_width: usize, target_height: usize, background: Pixel) {
        let mut new = Frame::new(target_width, target_height, background);
        let (x, y) = (target_width/2-(self.width/2), target_height/2-(self.height/2));

        for x2 in 0..self.width {
            for y2 in 0..self.height {
                new.put_pixel(x2+x, y2+y, self.get_pixel(x2,y2));
            }
        }

        self.width = new.width;
        self.height = new.height;
        self.pixels = new.pixels;
    }

    /// Tints the image with the Pixel colour provided and strength provided. Strength is 0-1, where one fully replaces the image with the colour and 0 keeps it the same.
    /// 
    /// # Arguments
    /// 
    /// * `target_color` - The colour to tint the image with.
    /// * `strength` - The strength of the tinting effect, between 0 and 1.
    pub fn tint(&mut self, target_color: Pixel, strength: f32) {
        for pixel in &mut self.pixels {
            pixel.r = lerp(strength, pixel.r as f32, target_color.r as f32) as u8;
            pixel.g = lerp(strength, pixel.g as f32, target_color.g as f32) as u8;
            pixel.b = lerp(strength, pixel.b as f32, target_color.b as f32) as u8;
        }
    }

    /// Turns the frame monochrome and removes all colour.
    pub fn monochrome(&mut self) {
        for pixel in &mut self.pixels {
            let avg = ((pixel.r as usize + pixel.g as usize + pixel.b as usize) / 3) as u8;

            pixel.r = avg;
            pixel.g = avg;
            pixel.b = avg;
        }
    }

    /// Layers another Frame on top of the current frame.
    /// 
    /// # Arguments
    /// 
    /// * `other` - The frame to be drawn over the current frame.
    /// * `x_offset` - The x-coordinate offset for the overlay.
    /// * `y_offset` - The y-coordinate offset for the overlay.
    pub fn draw_over(&mut self, other: &Frame, x_offset: usize, y_offset: usize) {
        for y in 0..other.height {
            for x in 0..other.width {
                // Calculate the target position on the current frame
                let target_x = x + x_offset;
                let target_y = y + y_offset;

                // Ensure we don't go out of bounds
                if target_x < self.width && target_y < self.height {
                    let other_pixel = other.get_pixel(x, y);
                    self.put_pixel(target_x, target_y, other_pixel);
                }
            }
        }
    }

    /// Does the same thing as `draw_over` but also removes the background color provided to add transparency.
    /// 
    /// # Arguments
    /// 
    /// * `other` - The frame to be drawn over the current frame.
    /// * `x_offset` - The x-coordinate offset for the overlay.
    /// * `y_offset` - The y-coordinate offset for the overlay.
    /// * `chroma_key` - The colour to be treated as transparent.
    /// * `tolerance` - The tolerance level for the chroma keying effect.
    pub fn draw_with_chroma_key(
        &mut self,
        other: &Frame,
        x_offset: usize,
        y_offset: usize,
        chroma_key: Pixel,
        tolerance: u8, // New: Add a tolerance level
    ) {
        for y in 0..other.height {
            for x in 0..other.width {
                let target_x = x + x_offset;
                let target_y = y + y_offset;

                if target_x < self.width && target_y < self.height {
                    let other_pixel = other.get_pixel(x, y);

                    // Check if the pixel is close to the chroma key (with tolerance)
                    let is_chroma_key = (chroma_key.r as i16 - other_pixel.r as i16).abs() <= tolerance as i16
                        && (chroma_key.g as i16 - other_pixel.g as i16).abs() <= tolerance as i16
                        && (chroma_key.b as i16 - other_pixel.b as i16).abs() <= tolerance as i16;

                    if !is_chroma_key {
                        self.put_pixel(target_x, target_y, other_pixel);
                    }
                }
            }
        }
    }
}

/// Takes a video file and an audio file and combines them together.
/// 
/// # Arguments
/// 
/// * `input_video` - The path to the input video file.
/// * `input_audio` - The path to the input audio file.
/// * `output_path` - The path where the output file will be saved.
/// 
/// # Returns
/// 
/// A `Result` indicating success or failure of the operation.
pub fn combine_video_and_audio(input_video: &str, input_audio: &str, output_path: &str, ffmpeg: &str) -> Result<(), String> {
    let output = Command::new(ffmpeg)
        .arg("-i")
        .arg(input_video)
        .arg("-i")
        .arg(input_audio)
        .arg("-c:v")
        .arg("copy")
        .arg("-c:a")
        .arg("aac")
        .arg("-map")
        .arg("0:v")
        .arg("-map")
        .arg("1:a")
        .arg(output_path)
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).to_string()),
        Err(err) => Err(err.to_string()),
    }
}

/// The main class for handling videos. A video is a list of frames, with a set width and height for consistency.
#[derive(Clone)]
pub struct Video {
    frames: Vec<Frame>, // The frames that make up the video
    pub width: usize,   // The width of the video
    pub height: usize   // The height of the video
}

impl Video {
    /// Creates a new Video instance with the specified width and height.
    /// 
    /// # Arguments
    /// 
    /// * `width` - The width of the video.
    /// * `height` - The height of the video.
    /// 
    /// # Returns
    /// 
    /// A new `Video` instance.
    pub fn new(width: usize, height: usize) -> Video {
        Video {
            width,
            height,
            frames: Vec::new()
        }
    }

    /// Creates a Video from a file, extracting frames using FFmpeg.
    /// 
    /// # Arguments
    /// 
    /// * `filename` - The path to the video file.
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the new `Video` or an error message.
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
                Frame::from_img(frame_path).ok()
            })
            .collect();

        if frames.is_empty() {
            drop_folder(temp);
            return Err("No frames were successfully loaded.".to_string());
        }

        let first_frame = &frames[0];
        let video = Video {
            width: first_frame.width,
            height: first_frame.height,
            frames,
        };

        drop_folder(temp);

        Ok(video)
    }

    /// Makes all the frames monochrome.
    pub fn monochrome(&mut self) {
        for frame in &mut self.frames {
            frame.monochrome();
        }
    }

    /// Returns the number of frames in the video.
    /// 
    /// # Returns
    /// 
    /// The length of the video in frames.
    pub fn length(&self) -> usize {
        self.frames.len()
    }

    /// Appends a frame to the video.
    /// 
    /// # Arguments
    /// 
    /// * `frame` - The frame to be added to the video.
    pub fn append_frame(&mut self, frame: Frame) {
        if frame.width == self.width && frame.height == self.height {
            self.frames.push(frame);
        } else {
            panic!("Frame width or size does not match the video size\nFrame: {}x{}\nVideo: {}x{}", frame.width, frame.height, self.width, self.height);
        }
    }

    /// Appends multiple frames to the video.
    /// 
    /// # Arguments
    /// 
    /// * `frames` - A box of frames to be added to the video.
    pub fn bulk_append_frame(&mut self, frames: Box<[Frame]>) {
        for frame in frames.iter() {
            if frame.width == self.width && frame.height == self.height {
                self.frames.push(frame.clone());
            } else {
                panic!("Frame width or size does not match the video size\nFrame: {}x{}\nVideo: {}x{}", frame.width, frame.height, self.width, self.height);
            }
        }
    }

    /// Crops the video to the specified dimensions.
    /// 
    /// # Arguments
    /// 
    /// * `x_start` - The x-coordinate to start cropping from.
    /// * `y_start` - The y-coordinate to start cropping from.
    /// * `crop_width` - The width of the cropped area.
    /// * `crop_height` - The height of the cropped area.
    pub fn crop(&mut self, x_start: usize, y_start: usize, crop_width: usize, crop_height: usize) {
        // Validate crop boundaries
        if x_start + crop_width > self.width || y_start + crop_height > self.height {
            panic!("Crop dimensions exceed video boundaries");
        }

        // Modify each frame to create a cropped version
        self.frames = self.frames.iter_mut().map(|frame| {
            let mut cropped_pixels = Vec::with_capacity(crop_width * crop_height);

            for y in y_start..(y_start + crop_height) {
                for x in x_start..(x_start + crop_width) {
                    cropped_pixels.push(frame.get_pixel(x, y));
                }
            }

            Frame {
                pixels: cropped_pixels,
                width: crop_width,
                height: crop_height
            }
        }).collect();

        // Update video dimensions
        self.width = crop_width;
        self.height = crop_height;
    }

    /// Appends a still frame to the video a specified number of times.
    /// 
    /// # Arguments
    /// 
    /// * `frame` - The frame to be added.
    /// * `amount` - The number of times to add the frame.
    pub fn append_still(&mut self, frame: Frame, amount: usize) {
        if frame.width == self.width && frame.height == self.height {
            for _i in 0..amount {
                self.frames.push(frame.clone());
            }
        } else {
            panic!("Frame width or size does not match the video size\nFrame: {}x{}\nVideo: {}x{}", frame.width, frame.height, self.width, self.height);
        }
    }

    /// Applies a fade-in effect to the video.
    /// 
    /// # Arguments
    /// 
    /// * `frame_duration` - The duration of the fade-in effect in frames.
    /// * `color` - The colour to fade in.
    /// * `position` - The position to apply the fade-in effect (start or end).
    pub fn fade_in(&mut self, frame_duration: usize, color: Pixel, position: VideoPosition) {
        let frame_indices: Vec<usize> = match position {
            VideoPosition::START => (0..frame_duration).rev().collect(),
            VideoPosition::END => ((self.length() - frame_duration)..self.length()).collect(),
        };

        for (i, &frame_index) in frame_indices.iter().enumerate() {
            if let Some(frame) = self.frames.get_mut(frame_index) {
                frame.tint(color, i as f32 / frame_duration as f32);
            }
        }
    }

    /// Splices the video to keep only the frames in the specified range.
    /// 
    /// # Arguments
    /// 
    /// * `start` - The starting index of the splice.
    /// * `end` - The ending index of the splice.
    pub fn splice(&mut self, start: usize, end: usize) {
        if start >= self.length() || end >= self.length() || start > end {
            panic!(
                "Invalid range for splicing: start={} end={} length={}",
                start, end, self.length()
            );
        }

        self.frames = self.frames[start..=end].to_vec();
    }

    /// Concatenates another video to the current video.
    /// 
    /// # Arguments
    /// 
    /// * `other_video` - The video to be concatenated.
    pub fn concat(&mut self, other_video: Video) {
        for frame in other_video.frames {
            self.frames.push(frame.clone());
        }
    }

    /// Saves the video to the specified location with the given frames per second (fps).
    /// 
    /// # Arguments
    /// 
    /// * `export_location` - The path where the video will be saved.
    /// * `fps` - The frames per second for the output video.
    pub fn save(&self, export_location: String, fps: u8, keep_folder: bool, ffmpeg: &str) {
        let temporary = create_tmp_folder();

        let progress_bar = indicatif::ProgressBar::new(self.frames.len() as u64);
        progress_bar.set_style(indicatif::ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} frames")
            .unwrap()
            .progress_chars("=> "));

        // Use Rayon to parallelize the loop
        self.frames.par_iter().enumerate().for_each(|(fi, frame)| {
            let mut img = RgbImage::new(frame.width as u32, frame.height as u32);
            for (i, pixel) in frame.pixels.iter().enumerate() {
                let y = i / frame.width;
                let x = i % frame.width;

                img.put_pixel(x as u32, y as u32, Rgb([pixel.r, pixel.g, pixel.b]));
            }
            img.save(format!("{}/image{}.bmp", temporary, fi + 1)).unwrap();
            progress_bar.inc(1);
        });

        progress_bar.finish();
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

    /// Retrieves a reference to a specific frame in the video.
    /// 
    /// # Arguments
    /// 
    /// * `frame_number` - The index of the frame to retrieve.
    /// 
    /// # Returns
    /// 
    /// A reference to the specified `Frame`.
    pub fn get_frame(&self, frame_number: usize) -> &Frame {
        &self.frames[frame_number]
    }

    /// Retrieves a mutable reference to a specific frame in the video.
    /// 
    /// # Arguments
    /// 
    /// * `frame_number` - The index of the frame to retrieve.
    /// 
    /// # Returns
    /// 
    /// A mutable reference to the specified `Frame`.
    pub fn get_frame_mut(&mut self, frame_number: usize) -> &mut Frame {
        &mut self.frames[frame_number]
    }

    /// Draws an overlay frame over a range of frames in the video.
    /// 
    /// # Arguments
    /// 
    /// * `overlay_frame` - The frame to overlay.
    /// * `x_offset` - The x-coordinate offset for the overlay.
    /// * `y_offset` - The y-coordinate offset for the overlay.
    /// * `start_frame` - The starting index of the frames to overlay on.
    /// * `end_frame` - The ending index of the frames to overlay on.
    pub fn bulk_draw_over(
        &mut self,
        overlay_frame: &Frame,
        x_offset: usize,
        y_offset: usize,
        start_frame: usize,
        end_frame: usize,
    ) {
        if start_frame > end_frame || end_frame >= self.length() {
            panic!(
                "Invalid range: start_frame={} end_frame={} length={}",
                start_frame, end_frame, self.length()
            );
        }

        for i in start_frame..=end_frame {
            self.frames[i].draw_over(overlay_frame, x_offset, y_offset);
        }
    }

    /// Draws an overlay frame over a range of frames in the video using chroma keying.
    /// 
    /// # Arguments
    /// 
    /// * `overlay_frame` - The frame to overlay.
    /// * `x_offset` - The x-coordinate offset for the overlay.
    /// * `y_offset` - The y-coordinate offset for the overlay.
    /// * `chroma_key` - The colour to be treated as transparent.
    /// * `start_frame` - The starting index of the frames to overlay on.
    /// * `end_frame` - The ending index of the frames to overlay on.
    /// * `tolerance` - The tolerance level for the chroma keying effect.
    pub fn bulk_draw_with_chroma_key(
        &mut self,
        overlay_frame: &Frame,
        x_offset: usize,
        y_offset: usize,
        chroma_key: Pixel,
        start_frame: usize,
        end_frame: usize,
        tolerance: u8
    ) {
        if start_frame > end_frame || end_frame >= self.length() {
            panic!(
                "Invalid range: start_frame={} end_frame={} length={}",
                start_frame, end_frame, self.length()
            );
        }

        for i in start_frame..=end_frame {
            self.frames[i].draw_with_chroma_key(overlay_frame, x_offset, y_offset, chroma_key, tolerance);
        }
    }
}
