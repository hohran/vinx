use clap::Parser;
use rsframe::vfx::video::{Video,Frame};

/// Simple program to grep a file or stdin
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// path to video to process
    pub video_path: String,

    /// path to vinx program
    pub program_path: String,
}

fn main() {
    let args = Args::parse();
    vinx::run(args.video_path, args.program_path);
    // let f = Frame::from_img("test.png".to_string()).unwrap();
    // let mut vid = Video::new(f.width, f.height);
    // vid.append_still(f, 300);
    // vid.save("white-long.mp4".to_string(), 24, false, "ffmpeg");
}
