use std::path::PathBuf;
use std::collections::HashMap;
use std::fs;

pub struct FileEntry {
    pub path: PathBuf
}

pub struct DirEntry {
    pub path: PathBuf,
    file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    expanded: bool
}

pub enum FileTreeEntry {
    File(FileEntry),
    Dir(DirEntry)
}

impl FileTreeEntry {
    fn new(path: PathBuf) -> FileTreeEntry {
        if path.is_dir() {
            FileTreeEntry::Dir(DirEntry { path, expanded: false, file_tree: HashMap::new() })
        } else {
            FileTreeEntry::File(FileEntry { path })
        }
    }
}

pub struct AppState {
    /// Directory of the entire project, can only be a single one
    /// It is either opened directly (like `love .`) or it is calculated
    /// based on the provided path
    pub working_directory: PathBuf,
    pub file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    pub file_content: String,
    cursor_x: i32,
    cursor_y: i32,
    editor_offset_x: i32,
    editor_offset_y: i32
}

impl AppState {
    pub fn new(file_content: String, working_directory: PathBuf) -> AppState {
        let mut app_state = AppState {
            working_directory,
            file_tree: HashMap::new(),
            file_content,
            cursor_x: 0,
            cursor_y: 0,
            editor_offset_x: 0,
            editor_offset_y: 0
        };

        app_state.read_directory(app_state.working_directory.clone());

        app_state
    }

    pub fn read_directory(&mut self, path: PathBuf) {
        // TODO: save the error as a directory state
        let dir_entries = fs::read_dir(&path).expect("Could not read directory");

        let values: Vec<FileTreeEntry> = dir_entries
            .filter_map(|result| result.ok())
            .map(|entry| FileTreeEntry::new(entry.path()))
            .collect();

        self.file_tree.insert(path, values);
            
    }
}