use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::ui_additions::ThemePreset;

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedState {
    pub last_file_path: Option<PathBuf>,
    pub current_index: usize,
    pub wpm: u32,
    pub last_theme: ThemePreset,
}

impl SavedState {
    fn data_path() -> Option<PathBuf> {
        dirs::data_dir().map(|mut path| {
            path.push("tfr");
            path.push("config.json");
            path
        })
    }
    pub fn load() -> Self {
        if let Some(path) = Self::data_path()
            && path.exists()
            && let Ok(content) = fs::read_to_string(path)
            && let Ok(state) = serde_json::from_str::<SavedState>(&content)
        {
            return state;
        }

        Self {
            last_file_path: None,
            current_index: 0,
            wpm: 300,
            last_theme: ThemePreset::OledBlack,
        }
    }
    pub fn save(&self) {
        if let Some(path) = Self::data_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, json);
            }
        }
    }
}
