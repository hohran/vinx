use std::process::exit;

use context::Context;
use translator::parser::parse;

use crate::{action::{ActionHandle, process_action_handles}, video::{VideoReader, VideoWriter}};

pub mod action;
pub mod event;
pub mod context;
pub mod video;
pub mod translator;
pub mod variable;

pub fn run(media_file: String, command_file: String, output_path: String) {
    let (mut stack, mut actions, operations, options) = match parse(&command_file) {
        Ok(x) => x,
        Err(e) => {
            e.print();
            exit(1);
        },
    };
    let mut action_handles: Vec<ActionHandle> = vec![];
    let mut ffmpeg_input = video::get_input(&media_file).unwrap();
    let mut reader = VideoReader::new(&mut ffmpeg_input);
    let mut writer = VideoWriter::from(&reader);
    let mut context = Context::new();
    context.set_reader(&mut reader);
    // let video = Video::from_file(media_file, "ffmpeg").expect("could not read video file");
    // let mut context = Context::from(video);
    // run the main loop
    'main_loop: while context.load_next_frame() {
        for i in 0..actions.len() {
            let a = &mut actions[i];
            a.step();
            a.trigger(&mut context, &mut stack, &operations, &mut action_handles);
            let should_stop = process_action_handles(&mut action_handles, &mut actions); // TODO: this has to be
                                                                       // changed if action_handle
                                                                       // could reorder actions
            if should_stop {
                break 'main_loop;
            }
        }
        if options.save_video {
            writer.append_frame(context.pop_current_frame()).expect("error: failed to append frame to the output");
        }
    }
    if options.save_video {
        writer.save(&output_path).expect("error: failed to save output video");
        eprintln!("Output saved as {output_path}");
    }
    // let video = context.get_video();
    // video.save(output_path.to_string(), 24, false, "ffmpeg");
}
