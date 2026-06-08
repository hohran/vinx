use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
enum FileLoadState {
    Loading,
    Finished,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileDependency {
    Acyclic,
    Recursive,
    Redundant,
}

impl FileDependency {
    pub fn is_recursive(&self) -> bool {
        *self == FileDependency::Recursive
    }

    pub fn is_redundant(&self) -> bool {
        *self == FileDependency::Redundant
    }
}

pub struct FileManager {
    filenames: Vec<PathBuf>,
    load_states: Vec<FileLoadState>,
    contents: Vec<String>,
}

impl FileManager {
    pub fn new(filename: &str) -> Option<Self> {
        let mut path = PathBuf::new();
        path.push(filename);
        if !path.is_file() {
            return None;
        }
        Some(Self { filenames: vec![path], load_states: vec![FileLoadState::Loading], contents: vec![String::new()] })
    }

    /// Notes the start of processing of a file and loads its contents
    /// Returns:
    /// - Acyclic: when loading this file is ok
    /// - Recursive: when this file is currently being loaded
    /// - Redundant: when this file has already been loaded
    ///
    /// # Example
    /// ```vinx
    /// // inside file1.vinx
    /// load "file2.vinx";  // Acyclic
    /// load "file2.vinx";  // Redundant
    /// load "file1.vinx";  // Recursive
    /// ```
    pub fn start(&mut self, filename: &str) -> Option<FileDependency> {
        let mut path = self.get_current_directory();
        path.push(filename);
        if !self.path_into_vinx_file(&mut path) {
            return None;
        }
        let dependency = self._start(path);
        self.load_file_contents();
        Some(dependency)
    }

    /// Transforms a `filepath` into path that contains vinx file.
    /// If filepath = 'foo', it would look if foo is a file, if so, it stays the same
    /// otherwise 'foo.vinx' would be tried
    /// Everything with .vinx suffix stays intact.
    fn path_into_vinx_file(&self, filepath: &mut PathBuf) -> bool {
        if filepath.is_file() {
            return true;
        }
        filepath.add_extension("vinx");
        filepath.is_file()
    }

    /// Returns the directory of currently processed file
    fn get_current_directory(&self) -> PathBuf {
        let mut path = self.get_current_pathbuf().clone();
        assert!(path.pop());
        path
    }

    /// Notes the start of loading file and returns its dependency on other files.
    /// Does not load contents of the file.
    fn _start(&mut self, filepath: PathBuf) -> FileDependency {
        let file_pos = self.filenames.iter().position(|n| *n == filepath);
        if let Some(file) = file_pos {
            match self.load_states[file] {
                FileLoadState::Loading => return FileDependency::Recursive,
                FileLoadState::Finished => return FileDependency::Redundant,
            }
        }
        self.filenames.push(filepath);
        self.load_states.push(FileLoadState::Loading);
        self.contents.push(String::new());
        FileDependency::Acyclic
    }

    pub fn load_file_contents(&mut self) -> &str {
        let i = self.current_file_index();
        let filename = &self.filenames[i];
        let contents = std::fs::read_to_string(filename).expect("error reading input file");
        self.contents[i] = contents;
        &self.contents[i]
    }

    /// Notes the finish of loading a file
    pub fn finish_file(&mut self) {
        let i = self.current_file_index();
        self.load_states[i] = FileLoadState::Finished;
    }

    /// Returns index of a currently processed file
    fn current_file_index(&self) -> usize {
        match self.load_states.iter().rev().position(|s| *s == FileLoadState::Loading) {
            Some(p) => self.filenames.len()-1-p,
            None => panic!("error: no file is currently loaded")
        }
    }

    /// Returns contents of a currently processed file
    pub fn current_file_contents(&self) -> &str {
        let i = self.current_file_index();
        &self.contents[i]
    }

    /// Returns the path buffer of currently processed file.
    fn get_current_pathbuf(&self) -> &PathBuf {
        let i = self.current_file_index();
        &self.filenames[i]
    }

    /// Returns the name of currently processed file if there is any.
    pub fn current_file(&self) -> &str {
        let i = self.current_file_index();
        self.filenames[i].to_str().expect("error: failed to get string interpretation of the current file")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_finish() {
        let mut fm = FileManager::new("examples/rows.vinx").unwrap();
        assert_eq!(fm.filenames[0],"examples/rows.vinx".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Loading);
        fm._start("file2".into());
        assert_eq!(fm.filenames[0],"examples/rows.vinx".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Loading);
        assert_eq!(fm.filenames[1],"file2".to_string());
        assert_eq!(fm.load_states[1],FileLoadState::Loading);
        fm.finish_file();
        assert_eq!(fm.filenames[0],"examples/rows.vinx".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Loading);
        assert_eq!(fm.filenames[1],"file2".to_string());
        assert_eq!(fm.load_states[1],FileLoadState::Finished);
        fm.finish_file();
        assert_eq!(fm.filenames[0],"examples/rows.vinx".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Finished);
        assert_eq!(fm.filenames[1],"file2".to_string());
        assert_eq!(fm.load_states[1],FileLoadState::Finished);
    }

    #[test]
    fn test_current_file_index() {
        let mut fm = FileManager::new("examples/rows.vinx").unwrap();
        assert_eq!(fm.current_file_index(), 0);
        fm._start("file2".into());
        assert_eq!(fm.current_file_index(), 1);
        fm.finish_file();
        assert_eq!(fm.current_file_index(), 0);
    }

    #[test]
    #[should_panic]
    fn test_current_file_index_panic() {
        let mut fm = FileManager::new("examples/rows.vinx").unwrap();
        fm.finish_file();
        fm.current_file_index();
    }

    #[test]
    fn test_current_file_name() {
        let mut fm = FileManager::new("examples/rows.vinx").unwrap();
        assert_eq!(fm.current_file(), "examples/rows.vinx");
        fm._start("file2".into());
        assert_eq!(fm.current_file(), "file2");
        fm.finish_file();
        assert_eq!(fm.current_file(), "examples/rows.vinx");
    }
}
