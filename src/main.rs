fn main() {
    // let frame = vinx::video::Frame::from_img("test.png".to_string()).unwrap();
    // let mut vid = vinx::Video::new(frame.width, frame.height);
    // vid.append_still(frame, 100);
    // vid.save("black.mp4".to_string(), 24, false, "ffmpeg");
    // let mut actions = vinx::get_actions("".to_string());
    // vinx::main_loop("white.mp4".to_string(), &mut actions);
    // Frame::new(600, 400).save("test.png");
    // vinx::translator::print_ast("in.vinx");
    vinx::run("white.mp4".to_string(), "in.vinx".to_string());
}
