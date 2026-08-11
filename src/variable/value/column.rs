use std::fmt::Display;

use image::Rgb;

use crate::video::Frame;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column (Vec<image::Rgb<u8>>, u32);

impl Column {
    pub fn get(&self) -> &Vec<image::Rgb<u8>> {
        &self.0
    }

    pub fn take(img: &Frame, at: u32) -> Self {
        let (width, height) = img.dimensions();
        assert!(at < width, "column index out of bounds");

        let raw = img.as_raw();
        let stride = width as usize * 3;
        let x = at as usize;
        let data = (0..height as usize)
            .map(|y| {
                let offset = y * stride + x * 3;
                Rgb([raw[offset], raw[offset + 1], raw[offset + 2]])
            })
        .collect();
        Self(data, at)
    }

    pub fn default() -> Self {
        Self(vec![], 0)
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "column at {}", self.1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row (Vec<image::Rgb<u8>>, u32);

impl Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "row at {}", self.1)
    }
}

impl Row {
    pub fn get(&self) -> &Vec<image::Rgb<u8>> {
        &self.0
    }

    pub fn take(img: &Frame, at: u32) -> Self {
        let (width, height) = img.dimensions();
        assert!(at < height, "row index out of bounds");

        let raw = img.as_raw();
        let stride = width as usize * 3;
        let row_start = at as usize * stride;
        let data = raw[row_start..row_start + stride]
            .chunks_exact(3)
            .map(|c| Rgb([c[0], c[1], c[2]]))
            .collect();
        Self(data, at)
    }

    pub fn default() -> Self {
        Self(vec![], 0)
    }
}
