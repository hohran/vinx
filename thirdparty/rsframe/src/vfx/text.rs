use std::process::Command;
use crate::vfx::tmp::rng_string;

pub fn create_text(text: String, font: String, color: String, width: usize, height: usize, magick: &str) -> String {
    let img_name = format!("{}.bmp", rng_string(32));
    let output = Command::new(magick)
        .args(&[
            "-background",
            "transparent",
            "-font",
            font.as_str(),
            "-size",
            format!("{}x{}",width,height).as_str(),
            "-fill",
            color.as_str(),
            "-gravity",
            "center",
            format!("label:{}", text).as_str(),
            img_name.as_str(),
        ])
        .output()
        .expect("Failed to execute ImageMagick command");

    if output.status.success() {
        println!("Image successfully created: output.png");
    } else {
        eprintln!(
            "Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    img_name
}
