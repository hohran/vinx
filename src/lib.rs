use std::process::exit;

use context::Context;
use translator::parse;

use crate::{action::{ActionHandle, process_action_handles}, video::Video};

pub mod action;
pub mod event;
pub mod context;
pub mod video;
pub mod translator;
pub mod variable;

pub fn run(media_file: String, command_file: String, output_path: String) {
    let (mut stack, mut actions, operations) = match parse(&command_file) {
        Ok(x) => x,
        Err(e) => {
            e.print();
            exit(1);
        },
    };
    let mut action_handles: Vec<ActionHandle> = vec![];
    let video = Video::from_file(media_file, "ffmpeg").expect("could not read video file");
    let mut context = Context::from(video);
    // run the main loop
    for _ in 1..context.get_video_length()+1 {
        context.step();
        for i in 0..actions.len() {
            let a = &mut actions[i];
            a.step();
            a.trigger(&mut context, &mut stack, &operations, &mut action_handles);
            process_action_handles(&mut action_handles, &mut actions); // TODO: this has to be
                                                                       // changed if action_handle
                                                                       // could reorder actions
        }
    }
    let video = context.get_video();
    video.save(output_path.to_string(), 24, false, "ffmpeg");
    eprintln!("Output saved as {output_path}");
}
