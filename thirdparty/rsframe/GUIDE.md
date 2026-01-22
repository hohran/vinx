# rsframe guide
## Requirements
 - ***ffmpeg***: ffmpeg is the main video editor component of `rsframe`. Windows users need to install a precompiled build, which you can find online, and then add it to PATH. Linux users can just run the `sudo apt install` command.
 - ***ImageMagick***: ImageMagick is used for creating text frames. You can leave ImageMagick uninstall if you're not going to do any text editing, however it is still recommended to install it. Go to their official website and install it. Windows users must also have to add it to path
### Basic Video Editing
In this short chapter, I will demonstrate how to turn a video monochrome.  
Firstly, let's get a basic project set up. In your project directory, there should be a video called `placeholder.mp4````rust
use rsframe::vfx::video;

fn main() {
    
}```
As you can see in this code block, the `video` module is where all the video editing tools are.
Let's load a video. To do this, we'll call the `from_file` method from the `Video` struct.
```rust
let mut vid = video::Video::from_file("placeholder.mp4".to_string(), "ffmpeg").expect("Cannot open video.");
```
This code will load a video into memory. You shouldn't do this with really long videos as loading in a video takes quite a bit of time.
```rust
let mut vid = video::Video::from_file("placeholder.mp4".to_string(), "ffmpeg").expect("Cannot open video.");

vid.monochrome();
vid.save("output.mp4".to_string(), 24, false, "ffmpeg");
```
Above is a code block demonstrating how to modify a Video and how to save it.
Saving a video takes three parameters:
1. Output path
2. FPS
3. Keep rendering folder  

Keeping the debug folder can be valuable if you're trying to figure what went wrong in your program and at what time.

### Text
Text editing requires you to have ImageMagick, and it is added to PATH if you're on Windows.
First of all, you can copy the empty template that we put up above into a project.  
You can paste the code block below into the main function.
```rust
let mut vid = video::Video::new(512, 512);
let sample_text = video::Frame::text(512, 512, "Arial".to_string(), "#fff".to_string(), "Hello, world!".to_string(), "magick").unwrap();

vid.append_still(sample_text, 100);
vid.save("out.mp4".to_string(), 24, false, "ffmpeg");
```
This code will create a video file with the text `Hello, world!` on it.
You can easily customize this. Here's an example that uses the users input to create the video.
```rust
use rsframe::vfx::video;

fn main() {
    let mut user_in = String::new();
    std::io::stdin().read_line(&mut user_in).expect("Could not read your input.");
    let write = user_in.trim().to_string();
    let mut vid = video::Video::new(512, 512);
    let sample_text = video::Frame::text(512, 512, "Arial".to_string(), "#fff".to_string(), write, "magick").unwrap();

    vid.append_still(sample_text, 100);
    vid.save("out.mp4".to_string(), 24, false, "ffmpeg");
}
```
*^ Final Code Product*  

### Drawing over Things
In this example, we will read user input, and then draw it as text over a background.
But firstly, we're going to need an image to draw over. I recommend sites like [Unsplash](https://unsplash.com/) because they're free for both commercial and personal use and also high quality.
Please do note that really high resolution images (above 2k!) will take an extremely long time to render with `rsframe`. You should check your images resolution before downloading. Otherwise, you can use a program like [paint.net](https://www.getpaint.net/) or [Photopea](https://www.photopea.com/) to resize the image.  
In the code example below, I am using a file called *"background.jpg"*; you might have a different name for it, so look out for that.
```rust
let mut user_in = String::new();
std::io::stdin().read_line(&mut user_in).expect("Could not read your input.");
let write = user_in.trim().to_string();
let mut image = video::Frame::from_img("background.jpg".to_string(), "ffmpeg").unwrap();
let sample_text = video::Frame::text(image.width, image.height, "Arial".to_string(), "#fff".to_string(), write, "magick").unwrap();
image.draw_with_chroma_key(&sample_text, 0, 0, video::Pixel {
    r: 0,
    g: 0,
    b: 0
}, 3);
let mut vid = video::Video::new(image.width, image.height); 
vid.append_still(image, 100);

vid.save("out.mp4".to_string(), 24, false, "ffmpeg");
```
The code above will generate a video based on the text the user inputted with a background behind it.
We can also give the text a drop shadow if we create a second text frame.
```rust
use rsframe::vfx::video;

fn main() {
    let mut user_in = String::new();
    std::io::stdin().read_line(&mut user_in).expect("Could not read your input.");
    let write = user_in.trim().to_string();
    let mut image = video::Frame::from_img("background.jpg".to_string()).unwrap();
    let sample_text = video::Frame::text(image.width, image.height, "Arial".to_string(), "#fff".to_string(), write.clone(), "magick").unwrap();
    let shadow = video::Frame::text(image.width, image.height, "Arial".to_string(), "#383839".to_string(), write, "magick").unwrap();
    image.draw_with_chroma_key(&shadow, 7, 7, video::Pixel {
        r: 0,
        g: 0,
        b: 0
    }, 3);
    image.draw_with_chroma_key(&sample_text, 0, 0, video::Pixel {
        r: 0,
        g: 0,
        b: 0
    }, 3);
    let mut vid = video::Video::new(image.width, image.height);
    vid.append_still(image, 100);

    vid.save("out.mp4".to_string(), 24, false, "ffmpeg");
}
```
The code above does the same thing as the one above it but adds a drop shadow to the text.

***TODO: Add more tutorials***  
But for right now, this and the `docs.rs` page should serve you well enough to figure out how to use `rsframe`.

