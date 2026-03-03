use std::{path::Path, process::Command};

// extern crate rsframe;
use rand::{Rng, distributions::Alphanumeric, thread_rng};
// use rsframe::vfx::video::Pixel;
// pub use rsframe::vfx::video::{Frame,Video};
use image::{ImageReader, ImageResult, Pixel, RgbImage};
use rayon::prelude::*;
use rayon::iter::IntoParallelRefIterator;
use crate::variable::values::{Color, Effect};
use std::fs;

pub type Frame = RgbImage;

pub struct Video {
    width: u32,
    height: u32,
    frames: Vec<Frame>
}

impl Video {
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


// ************* Drawable impl ************* //
pub trait Drawable {
    fn draw_rect(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Color);
    fn draw_rect_outline(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Color);
    fn draw_effect_rect(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), e: Effect);
}

impl Drawable for Frame {
    /// Draws a rectangle filled with p
    fn draw_rect(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Color) {
        let width = self.width();
        let height = self.height();
        let l = top_left.0 as u32 % width;
        let r = bottom_right.0 as u32 % width;
        let t = top_left.1 as u32 % height;
        let b = bottom_right.1 as u32 % height;
        // draw top/bottom row
        if l <= r {
            if t <= b {
                for y in t..b {
                    for x in l..r { self.put_pixel(x, y, p); }
                }
            } else {
                for y in 0..b {
                    for x in l..r { self.put_pixel(x, y, p); }
                }
                for y in t..height {
                    for x in l..r { self.put_pixel(x, y, p); }
                }
            }
        } else {
            if t <= b {
                for y in t..b {
                    for x in 0..r { self.put_pixel(x, y, p); }
                    for x in l..width { self.put_pixel(x, y, p); }
                }
            } else {
                for y in 0..b {
                    for x in 0..r { self.put_pixel(x, y, p); }
                    for x in l..width { self.put_pixel(x, y, p); }
                }
                for y in t..height {
                    for x in 0..r { self.put_pixel(x, y, p); }
                    for x in l..width { self.put_pixel(x, y, p); }
                }
            }
        }
    }

    fn draw_effect_rect(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), e: Effect) {
        let width = self.width();
        let height = self.height();
        let l = top_left.0 as u32 % width;
        let r = bottom_right.0 as u32 % width;
        let t = top_left.1 as u32 % height;
        let b = bottom_right.1 as u32 % height;
        match e {
            Effect::Blur => blur(self, l, r, t, b),
            Effect::Random => randomize(self, l, r, t, b),
            Effect::Inverse => inverse(self, l, r, t, b),
        }
    }


    fn draw_rect_outline(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Color) {
        let width = self.width();
        let height = self.height();
        let l = top_left.0 as u32 % width;
        let r = bottom_right.0 as u32 % width;
        let t = top_left.1 as u32 % height;
        let b = bottom_right.1 as u32 % height;
        // draw top/bottom row
        if l <= r {
            for x in l..r {
                self.put_pixel(x, t, p);
                self.put_pixel(x, b, p);
            }
        } else {
            for x in 0..r {
                self.put_pixel(x, t, p);
                self.put_pixel(x, b, p);
            }
            for x in l..width {
                self.put_pixel(x, t, p);
                self.put_pixel(x, b, p);
            }
        }
        // draw edges
        if t <= b {
            for y in t..b {
                self.put_pixel(l, y, p);
                self.put_pixel(r, y, p);
            }
        } else {
            for y in 0..b {
                self.put_pixel(l, y, p);
                self.put_pixel(r, y, p);
            }
            for y in t..height {
                self.put_pixel(l, y, p);
                self.put_pixel(r, y, p);
            }
        }
    }
}

fn blur(_f: &mut Frame, _l: u32, _r: u32, _t: u32, _b: u32) {
    todo!();
}

fn randomize(f: &mut Frame, l: u32, r: u32, t: u32, b: u32) {
    let width = f.width();
    let height = f.height();
    let _rng = thread_rng();
    if l <= r {
        let mut rng = _rng.clone();
        let mut x_gen = std::iter::repeat_with(move || rng.gen_range(l..=r));
        if t <= b {
            let mut rng = _rng.clone();
            let mut y_gen = std::iter::repeat_with(move || rng.gen_range(t..=b));
            for y in t..b {
                for x in l..r {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
            }
        } else {
            let mut rng = _rng.clone();
            let mut y_gen = std::iter::repeat_with(move || rng.gen_range(b..=l+height) % height);
            for y in 0..b {
                for x in l..r {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
            }
            for y in t..height {
                for x in l..r {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
            }
        }
    } else {
        let mut rng = _rng.clone();
        let mut x_gen = std::iter::repeat_with(move || rng.gen_range(r..=l+width) % width);
        if t <= b {
            let mut rng = _rng.clone();
            let mut y_gen = std::iter::repeat_with(move || rng.gen_range(t..=b));
            for y in t..b {
                for x in 0..r { 
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
                for x in l..width { 
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
            }
        } else {
            let mut rng = _rng.clone();
            let mut y_gen = std::iter::repeat_with(move || rng.gen_range(b..=l+height) % height);
            for y in 0..b {
                for x in 0..r { 
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
                for x in l..width {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
            }
            for y in t..height {
                for x in 0..r {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
                for x in l..width {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, *p);
                }
            }
        }
    }
}

fn inverse(f: &mut Frame, l: u32, r: u32, t: u32, b: u32) {
    fn inverse_pixel(f: &mut Frame, x: u32, y: u32) {
        f.get_pixel_mut(x, y).invert();
    }

    let width = f.width();
    let height = f.height();
    if l <= r {
        if t <= b {
            for y in t..b {
                for x in l..r {
                    inverse_pixel(f, x, y);
                }
            }
        } else {
            for y in 0..b {
                for x in l..r { inverse_pixel(f, x, y); }
            }
            for y in t..height {
                for x in l..r { inverse_pixel(f, x, y); }
            }
        }
    } else {
        if t <= b {
            for y in t..b {
                for x in 0..r { inverse_pixel(f, x, y); }
                for x in l..width { inverse_pixel(f, x, y); }
            }
        } else {
            for y in 0..b {
                for x in 0..r { inverse_pixel(f, x, y); }
                for x in l..width { inverse_pixel(f, x, y); }
            }
            for y in t..height {
                for x in 0..r { inverse_pixel(f, x, y); }
                for x in l..width { inverse_pixel(f, x, y); }
            }
        }
    }
}
