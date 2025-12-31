extern crate rsframe;
use rand::{thread_rng, Rng};
use rsframe::vfx::video::Pixel;
pub use rsframe::vfx::video::{Frame,Video};

use crate::variable::values::Effect;

pub trait Drawable {
    fn draw_rect(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Pixel);
    fn draw_rect_outline(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Pixel);
    fn draw_effect_rect(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), e: Effect);
}

impl Drawable for Frame {
    /// Draws a rectangle filled with p
    fn draw_rect(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Pixel) {
        let width = self.width;
        let height = self.height;
        let l = top_left.0 % self.width;
        let r = bottom_right.0 % self.width;
        let t = top_left.1 % self.height;
        let b = bottom_right.1 % self.height;
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
        let l = top_left.0 % self.width;
        let r = bottom_right.0 % self.width;
        let t = top_left.1 % self.height;
        let b = bottom_right.1 % self.height;
        match e {
            Effect::Blur => blur(self, l, r, t, b),
            Effect::Random => randomize(self, l, r, t, b),
            Effect::Inverse => inverse(self, l, r, t, b),
        }
    }


    fn draw_rect_outline(&mut self, top_left: (usize,usize), bottom_right: (usize,usize), p: Pixel) {
        let width = self.width;
        let height = self.height;
        let l = top_left.0 % self.width;
        let r = bottom_right.0 % self.width;
        let t = top_left.1 % self.height;
        let b = bottom_right.1 % self.height;
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

fn blur(f: &mut Frame, l: usize, r: usize, t: usize, b: usize) {
    todo!();
}

fn randomize(f: &mut Frame, l: usize, r: usize, t: usize, b: usize) {
    let width = f.width;
    let height = f.height;
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
                    f.put_pixel(x, y, p);
                }
            }
        } else {
            let mut rng = _rng.clone();
            let mut y_gen = std::iter::repeat_with(move || rng.gen_range(b..=l+height) % height);
            for y in 0..b {
                for x in l..r {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, p);
                }
            }
            for y in t..height {
                for x in l..r {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, p);
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
                    f.put_pixel(x, y, p); 
                }
                for x in l..width { 
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, p); 
                }
            }
        } else {
            let mut rng = _rng.clone();
            let mut y_gen = std::iter::repeat_with(move || rng.gen_range(b..=l+height) % height);
            for y in 0..b {
                for x in 0..r { 
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, p);
                }
                for x in l..width {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, p);
                }
            }
            for y in t..height {
                for x in 0..r {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, p);
                }
                for x in l..width {
                    let p = f.get_pixel(x_gen.next().unwrap(), y_gen.next().unwrap());
                    f.put_pixel(x, y, p);
                }
            }
        }
    }
}

fn inverse(f: &mut Frame, l: usize, r: usize, t: usize, b: usize) {
    fn inverse_pixel(f: &mut Frame, x: usize, y: usize) {
        let mut p = f.get_pixel(x, y);
        p.r = u8::MAX - p.r;
        p.g = u8::MAX - p.g;
        p.b = u8::MAX - p.b;
        f.put_pixel(x, y, p);
    }

    let width = f.width;
    let height = f.height;
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
