<!-- [![MIT License][license-shield]][license-url] -->

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <!-- <a href="/blob/main/LICENSE.txt"> -->
  <!--   <img src="images/logo.png" alt="Logo" width="80" height="80"> -->
  <!-- </a> -->

  <h3 align="center">vinx</h3>

  <p align="center">
    A video editing programming language
    <!-- <br /> -->
    <!-- <a href="https://github.com/hohran/vinx"><strong>Explore the docs »</strong></a> -->
    <br />
    <br />
    <a href="https://github.com/hohran/vinx">View Demo</a>
    &middot;
    <a href="https://github.com/hohran/vinx/issues/new?template=bug_report.md">Report Bug</a>
    &middot;
    <a href="https://github.com/hohran/vinx/issues/new?template=feature_request.md">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<!-- <details> -->
<!--   <summary>Table of Contents</summary> -->
<!--   <ol> -->
<!--     <li> -->
<!--       <a href="#about-the-project">About The Project</a> -->
<!--     </li> -->
<!--     <li> -->
<!--       <a href="#getting-started">Getting Started</a> -->
<!--       <ul> -->
<!--         <li><a href="#prerequisites">Prerequisites</a></li> -->
<!--         <li><a href="#installation">Installation</a></li> -->
<!--       </ul> -->
<!--     </li> -->
<!--     <li><a href="#usage">Usage</a></li> -->
<!--     <!-- <li><a href="#roadmap">Roadmap</a></li> -->
<!--     <li><a href="#contributing">Contributing</a></li> -->
<!--     <li><a href="#license">License</a></li> -->
<!--     <!-- <li><a href="#contact">Contact</a></li> -->
<!--     <!-- <li><a href="#acknowledgments">Acknowledgments</a></li> -->
<!--   </ol> -->
<!-- </details> -->

<!-- ABOUT THE PROJECT -->
## About The Project

<!-- [![Product Name Screen Shot][product-screenshot]](https://example.com) -->

*vinx* is a Rust-based video editing programming language.
It's main design principle is to be easy to use and elegant.
That means that you can comfortably use it without a mouse and GUI.

<!-- GETTING STARTED -->
## Getting Started

Since vinx is only a passion project, it depends on technologies, built by way smarter and well-paid brains.
To set it up, you will need the following:

### Prerequisites

Install these technologies for this project to work properly:
* [FFmpeg](https://ffmpeg.org/download.html)
* [Rust](https://rust-lang.org/learn/get-started/)

Check installation with:
```bash
ffmpeg
cargo --version
```

### Installation

I am keeping this project still very personal, so it is not submitted anywhere.
For the time being.

1. Install the dependencies
2. Build this repo from source with
```bash
cargo build --release
```
3. Use the `vinx` executable found in `target/release/`

<!-- USAGE EXAMPLES -->
## Usage

```bash
Usage: vinx [OPTIONS] <VIDEO_PATH> <PROGRAM_PATH> [OUTPUT_PATH]

Arguments:
  [VIDEO_PATH]    path to video to process; it can be in most of the traditional formats
  [PROGRAM_PATH]  path to vinx program, usually with .vinx suffix
  [OUTPUT_PATH]   path to the output; defaults to "out.mp4"

Options:
  -l, --list     list all possible events
  -h, --help     Print help
  -V, --version  Print version
```

See `examples/` to have a nice walkthrough for all the features.

You can then start to write your own `.vinx` files.


### Syntax highlighting

You can install the vinx grammar from [tree-sitter-vinx](https://github.com/hohran/tree-sitter-vinx) and integrate it into your favorite text editor.

For VS Code, this may not work, as far as I am concerned.
<!-- Use this space to show useful examples of how a project can be used. Additional screenshots, code examples and demos work well in this space. You may also link to more resources. -->
<!---->
<!-- _For more examples, please refer to the [Documentation](https://example.com)_ -->

<!-- ROADMAP -->
<!-- ## Roadmap -->
<!---->
<!-- - [x] Add Changelog -->
<!-- - [x] Add back to top links -->
<!-- - [ ] Add Additional Templates w/ Examples -->
<!-- - [ ] Add "components" document to easily copy & paste sections of the readme -->
<!-- - [ ] Multi-language Support -->
<!--     - [ ] Chinese -->
<!--     - [ ] Spanish -->

<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

But, **I do not have much experience with managing open-source projects**.
So this is a check-list for contributions stolen from [Best-README-Template](https://github.com/othneildrew/Best-README-Template).

If you have a suggestion that would make vinx better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<!-- LICENSE -->
## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[license-shield]: https://img.shields.io/badge/license-MIT-brightgreen?style=for-the-badge
[license-url]: https://github.com/hohran/vinx/blob/main/LICENSE.txt
<!-- [linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555 -->
<!-- [linkedin-url]: https://linkedin.com/in/othneildrew -->
<!-- [product-screenshot]: images/screenshot.png -->
