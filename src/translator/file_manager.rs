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
}

pub struct FileManager {
    filenames: Vec<PathBuf>,
    load_states: Vec<FileLoadState>,
    contents: Vec<String>,
}

impl FileManager {
    pub fn new() -> Self {
        Self { filenames: vec![], load_states: vec![], contents: vec![] }
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
    pub fn start(&mut self, filename: &str) -> FileDependency {
        let mut path = self.get_current_directory();
        path.push(filename);
        self.path_into_vinx_file(&mut path);
        let dependency = self._start(path);
        self._load_file_contents();
        dependency
    }

    /// Transforms a `filepath` into path that contains vinx file.
    /// If filepath = 'foo', it would look if foo is a file, if so, it stays the same
    /// otherwise 'foo.vinx' would be tried
    /// Everything with .vinx suffix stays intact.
    fn path_into_vinx_file(&self, filepath: &mut PathBuf) {
        if filepath.is_file() {
            return;
        }
        filepath.add_extension("vinx");
        if !filepath.is_file() {
            panic!("error: {} does not exist", filepath.to_str().unwrap());
        }
    }

    /// Returns the directory of currently processed file
    fn get_current_directory(&self) -> PathBuf {
        match self.get_current_pathbuf() {
            Some(path) => {
                let mut path_clone = path.clone();
                assert!(path_clone.pop());
                path_clone
            }
            None => PathBuf::new()
        }
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

    fn _load_file_contents(&mut self) {
        let file_pos = self.current_file_index();
        if let Some(file) = file_pos {
            let filename = &self.filenames[file];
            let contents = std::fs::read_to_string(filename).expect("error reading input file");
            self.contents[file] = contents;
        } else {
            panic!("error: no currently processed file");
        }
    }

    /// Notes the finish of loading a file
    pub fn finish_file(&mut self) {
        let file_pos = self.current_file_index();
        if let Some(file) = file_pos {
            self.load_states[file] = FileLoadState::Finished;
        } else {
            panic!("error: tried to finish, but no file started loading");
        }
    }

    /// Returns index of a currently processed file
    fn current_file_index(&self) -> Option<usize> {
        match self.load_states.iter().rev().position(|s| *s == FileLoadState::Loading) {
            Some(p) => Some(self.filenames.len()-1-p),
            None => None
        }
    }

    /// Returns contents of a currently processed file
    pub fn current_file_contents(&self) -> Option<&str> {
        let file_pos = self.current_file_index();
        if let Some(file) = file_pos {
            Some(&self.contents[file])
        } else {
            None
        }
    }

    /// Returns the path buffer of currently processed file.
    fn get_current_pathbuf(&self) -> Option<&PathBuf> {
        self.current_file_index().map(|pos| &self.filenames[pos])
    }

    /// Returns the name of currently processed file if there is any.
    pub fn current_file(&self) -> Option<&str> {
        let file_pos = self.current_file_index();
        if let Some(file) = file_pos {
            self.filenames[file].to_str()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_finish() {
        let mut fm = FileManager::new();
        fm._start("file1".into());
        assert_eq!(fm.filenames[0],"file1".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Loading);
        fm._start("file2".into());
        assert_eq!(fm.filenames[0],"file1".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Loading);
        assert_eq!(fm.filenames[1],"file2".to_string());
        assert_eq!(fm.load_states[1],FileLoadState::Loading);
        fm.finish_file();
        assert_eq!(fm.filenames[0],"file1".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Loading);
        assert_eq!(fm.filenames[1],"file2".to_string());
        assert_eq!(fm.load_states[1],FileLoadState::Finished);
        fm.finish_file();
        assert_eq!(fm.filenames[0],"file1".to_string());
        assert_eq!(fm.load_states[0],FileLoadState::Finished);
        assert_eq!(fm.filenames[1],"file2".to_string());
        assert_eq!(fm.load_states[1],FileLoadState::Finished);
    }

    #[test]
    fn test_current_file_index() {
        let mut fm = FileManager::new();
        assert_eq!(fm.current_file_index(), None);
        fm._start("file1".into());
        assert_eq!(fm.current_file_index(), Some(0));
        fm._start("file2".into());
        assert_eq!(fm.current_file_index(), Some(1));
        fm.finish_file();
        assert_eq!(fm.current_file_index(), Some(0));
        fm.finish_file();
        assert_eq!(fm.current_file_index(), None);
    }

    #[test]
    fn test_current_file_name() {
        let mut fm = FileManager::new();
        assert_eq!(fm.current_file(), None);
        fm._start("file1".into());
        assert_eq!(fm.current_file(), Some("file1"));
        fm._start("file2".into());
        assert_eq!(fm.current_file(), Some("file2"));
        fm.finish_file();
        assert_eq!(fm.current_file(), Some("file1"));
        fm.finish_file();
        assert_eq!(fm.current_file(), None);
    }
}
