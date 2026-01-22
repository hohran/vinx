use std::process::Command;
use std::path::Path;

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
