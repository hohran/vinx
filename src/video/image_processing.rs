use super::*;
use rand::{Rng, thread_rng};
use image::{Pixel, Rgb, RgbImage};
use crate::variable::{Color, Effect};

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

// ************* Extendable impl ************* //
pub trait Extendable {
    fn append_column(&mut self, column: &[Rgb<u8>]);
    fn prepend_column(&mut self, column: &[Rgb<u8>]);
    fn append_row(&mut self, row: &[Rgb<u8>]);
    fn prepend_row(&mut self, row: &[Rgb<u8>]);
}

impl Extendable for Frame {
    /// Append a column to the right edge of the image.
    fn append_column(&mut self, column: &[Rgb<u8>]) {
        let (width, height) = self.dimensions();
        if width == 0 {
            let raw: Vec<u8> = column.iter().flat_map(|p| p.0).collect();
            *self = RgbImage::from_raw(1, column.len() as u32, raw).unwrap();
            return
        }
        assert_eq!(column.len() as u32, height, "column length must match image height");

        let old_raw = self.as_raw();
        let stride = width as usize * 3;
        let mut new_raw = Vec::with_capacity((width + 1) as usize * height as usize * 3);

        for y in 0..height as usize {
            new_raw.extend_from_slice(&old_raw[y * stride..(y + 1) * stride]);
            new_raw.extend_from_slice(&column[y].0);
        }

        *self = RgbImage::from_raw(width + 1, height, new_raw).unwrap()
    }

    /// Prepend a column to the left edge of the image.
    fn prepend_column(&mut self, column: &[Rgb<u8>]) {
        let (width, height) = self.dimensions();
        if width == 0 {
            let raw: Vec<u8> = column.iter().flat_map(|p| p.0).collect();
            *self = RgbImage::from_raw(1, column.len() as u32, raw).unwrap();
            return
        }
        assert_eq!(column.len() as u32, height, "column length must match image height");

        let old_raw = self.as_raw();
        let stride = width as usize * 3;
        let mut new_raw = Vec::with_capacity((width + 1) as usize * height as usize * 3);

        for y in 0..height as usize {
            new_raw.extend_from_slice(&column[y].0);
            new_raw.extend_from_slice(&old_raw[y * stride..(y + 1) * stride]);
        }

        *self = RgbImage::from_raw(width + 1, height, new_raw).unwrap()
    }

    /// Append a row to the bottom edge of the image.
    fn append_row(&mut self, row: &[Rgb<u8>]) {
        let (width, height) = self.dimensions();
        if height == 0 {
            let raw: Vec<u8> = row.iter().flat_map(|p| p.0).collect();
            *self = RgbImage::from_raw(row.len() as u32, 1, raw).unwrap();
            return
        }
        assert_eq!(row.len() as u32, width, "row length must match image width");

        let old_raw = self.as_raw();
        let mut new_raw = Vec::with_capacity(width as usize * (height + 1) as usize * 3);

        new_raw.extend_from_slice(old_raw);
        for pixel in row {
            new_raw.extend_from_slice(&pixel.0);
        }

        *self = RgbImage::from_raw(width, height + 1, new_raw).unwrap()
    }

    /// Prepend a row to the top edge of the image.
    fn prepend_row(&mut self, row: &[Rgb<u8>]) {
        let (width, height) = self.dimensions();
        if height == 0 {
            let raw: Vec<u8> = row.iter().flat_map(|p| p.0).collect();
            *self = RgbImage::from_raw(row.len() as u32, 1, raw).unwrap();
            return
        }
        assert_eq!(row.len() as u32, width, "row length must match image width");

        let old_raw = self.as_raw();
        let mut new_raw = Vec::with_capacity(width as usize * (height + 1) as usize * 3);

        for pixel in row {
            new_raw.extend_from_slice(&pixel.0);
        }
        new_raw.extend_from_slice(old_raw);

        *self = RgbImage::from_raw(width, height + 1, new_raw).unwrap()
    }
}
