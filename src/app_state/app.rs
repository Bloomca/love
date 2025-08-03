use ratatui::style::Color;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::editor::{FileTreeEntry, UIState};

pub struct AppState {
    /// Directory of the entire project, can only be a single one
    /// It is either opened directly (like `love .`) or it is calculated
    /// based on the provided path
    pub working_directory: PathBuf,
    pub file_tree: HashMap<PathBuf, Vec<FileTreeEntry>>,
    pub ui_state: UIState,
    pub config: Config,
    pub theme: Theme,
}

pub struct Config {
    pub tabs_to_spaces: bool,
    pub whitespaces_amount: usize,
}

impl Config {
    pub fn new() -> Self {
        Config {
            tabs_to_spaces: true,
            whitespaces_amount: 4,
        }
    }
}

pub struct Theme {
    pub bg_color: Color,
    pub status_bar_color: Color,
}

impl Theme {
    pub fn new() -> Self {
        Theme {
            bg_color: Color::Rgb(21, 6, 3),
            status_bar_color: Color::Rgb(30, 9, 20),
        }
    }
}

impl AppState {
    pub fn new(file_content: String, working_directory: PathBuf) -> AppState {
        let lines_number = file_content.lines().count();
        let lines: Vec<Vec<char>> = file_content.lines().map(|s| s.chars().collect()).collect();

        AppState {
            working_directory,
            file_tree: HashMap::new(),
            ui_state: UIState::new(lines_number.to_string().len(), lines),
            config: Config::new(),
            theme: Theme::new(),
        }
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
