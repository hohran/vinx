use std::collections::HashMap;

use context::Context;
pub use rsframe::vfx::video::{Frame, Pixel, Video};
use translator::parse;

pub mod action;
pub mod event;
pub mod context;
pub mod video;
pub mod translator;
pub mod variable;

fn get_action_activeness(actions: &Vec<action::Action>) -> HashMap<String,bool> {
    let mut action_activeness: HashMap<String,bool> = HashMap::new();
    for a in actions {
        println!("action: {}", a.get_name());
        if action_activeness.insert(a.get_name().to_string(), a.default_activeness()).is_some() {
            if !a.get_name().is_empty() {
                panic!("multiple occurences of action name {}",a.get_name());
            }
        }
    }
    action_activeness
}

pub fn run(media_file: String, command_file: String) {
    let (mut globals, mut components, mut actions, operations) = parse(&command_file);
    println!("--- operations ---");
    for op in operations.iter() {
        println!("{:?}", op);
    }
    println!("--- actions ---");
    for op in actions.iter() {
        println!("{}", op);
    }
    // return;
    let mut action_activeness = get_action_activeness(&actions);
    dbg!(&action_activeness);
    // return;
    let mut video = Video::from_file(media_file, "ffmpeg").expect("could not read video file");
    preprocess(&mut video);
    let mut context = Context::from(video);
    // run the main loop
    for _ in 1..context.get_video_length()+1 {
        context.step();
        for a in actions.iter_mut() {
            if !a.is_active(&action_activeness) {
                continue;
            }
            a.step(1);     // TODO derive millis from framerate
            a.trigger(&mut context, &mut globals, &mut components, &mut action_activeness, &operations);
        }
    }
    for _c in components.values_mut() {
        // c.step(1, &mut context);
    }
    let video = context.get_video();
    video.save("out.mkv".to_string(), 24, false, "ffmpeg");
}

pub fn preprocess(_video: &mut Video) {
}
