use clap::Parser;

/// Simple program to grep a file or stdin
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// path to video to process; it can be in most of the traditional formats.
    #[arg(required_unless_present = "list")]
    pub video_path: Option<String>,

    /// path to vinx program, usually with .vinx suffix.
    #[arg(required_unless_present = "list")]
    pub program_path: Option<String>,

    /// path to the output; defaults to "out.mp4"
    pub output_path: Option<String>,

    /// list all possible events
    #[arg(short, long)]
    pub list: bool,
}

fn main() {
    let args = Args::parse();
    
    if args.list {
        println!("list of possible events:
  restricted move Pos Dir by Int                - move a position, but stay in bounds of the frame
  move Pos Dir by Int                           - move the position and wrap around
  draw Color rectangle outline from Pos to Pos  - draw a colored outline
  draw Color rectangle from Pos to Pos          - draw filled rectangle
  draw Effect rectangle from Pos to Pos         - put an effect in the rectangle
  activate Str                                  - activate action with that name
  deactivate Str                                - deactivate action with that name
  set Any(1) to Any(1)                          - set a variable to a value of the same type
  rotate [Any(1)] Dir by Int                    - rotate a vector to left or right by certain step
  top [Any(1)] into Any(1)                      - put the top-most element of a vector into a variable
  add Int to Int                                - add some number to a variable
  toggle Str                                    - toggle an action with that name
  sub Int from Int                              - subtract some number from a variable");
        return;
    }
    let Some(video_path) = args.video_path else {
        panic!("")
    };
    let Some(program_path) = args.program_path else {
        panic!("")
    };
    vinx::run(video_path, program_path, args.output_path.unwrap_or("out.mp4".to_string()));
}
