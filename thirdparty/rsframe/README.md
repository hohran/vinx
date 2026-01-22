# rsframe

***IT IS HIGHLY RECOMMENDED THAT YOU READ THROUGH THE [GUIDE](https://github.com/khaki-git/rsframe/blob/master/GUIDE.md) BEFORE YOU START USING rsframe***

`rsframe` is a Rust library for video frame manipulation. It provides functionalities to tint frames, splice videos, concatenate videos, save videos, and retrieve specific frames. This library is designed to be efficient and easy to use, leveraging parallel processing where possible.

## Features

- **Tint Frames**: Apply a tint to specific frames in a video.
- **Splice Videos**: Keep only the frames within a specified range.
- **Concatenate Videos**: Combine two videos into one.
- **Save Videos**: Export the video to a specified location with a given frame rate.
- **Retrieve Frames**: Access specific frames in the video, either as immutable or mutable references.

## Usage

### Cargo
You can add `rsframe` to your project by executing the command
```shell
cargo add rsframe
```
or by adding `rsframe = "VERSION"` to your `Cargo.toml` file.