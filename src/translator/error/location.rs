use std::{fmt::Display, io::BufRead};

use tree_sitter::Range;

pub struct Location {
    filepath: String,
    range: Range,
}

impl Location {
    pub fn new(filepath: &str, range: Range) -> Self {
        return Self { filepath: filepath.to_string(), range }
    }

    fn get_loc(&self, max_row_width: usize) -> String {
        format!(" {:max_row_width$}--> {self}", " ")
    }

    fn fill(&self, max_row_width: usize) -> String {
        format!(" {:max_row_width$} |\n", " ")
    }

    // TODO: trim long lines
    fn line(&self, line: &str, _max_len: usize) -> String {
        format!(" {line}")
    }

    fn source(&self, max_row_width: usize, max_len: usize) -> String {
        let mut out = String::new();
        let start_row = self.range.start_point.row;
        let Ok(file) = std::fs::File::open(&self.filepath) else {
            panic!("error: failed to open file `{}`", self.filepath)
        };
        let source_lines = std::io::BufReader::new(file).lines().skip(start_row).take(self.range.end_point.row - start_row + 1);
        for (i, line) in source_lines.enumerate() {
            let Ok(line) = line else { panic!("error: unexpected EOF while reading file `{}`", self.filepath) };
            out += &format!(" {:>max_row_width$} |", start_row+i+1); // row is 0-indexed, but user usually
                                                                       // treats it as 1-indexed
            out += &self.line(&line, max_len);
            out += "\n";
        }
        out
    }

    pub fn get_source(&self) -> String {
        let max_len = 80; // maximum number of characters in the printed string
        let max_row_width = ((self.range.end_point.row+1).ilog10()+1) as usize;
        let loc = self.get_loc(max_row_width) + "\n";
        let fill = self.fill(max_row_width);
        let source = self.source(max_row_width, max_len);
        let out = loc + &fill + &source + &fill;
        out
    }

    pub fn get_concise_source(&self) -> String {
        let max_len = 80; // maximum number of characters in the printed string
        let max_row_width = ((self.range.end_point.row+1).ilog10()+1) as usize;
        let fill = self.fill(max_row_width);
        let source = self.source(max_row_width, max_len);
        let out = String::new() + &fill + &source + &fill;
        out
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.range.start_point;
        write!(f, "{}:{}:{}", self.filepath, start.row+1, start.column)
    }
}
