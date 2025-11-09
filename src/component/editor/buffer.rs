use crate::component::component_utils::new_clipboard;
use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::eyre::eyre;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use tui_textarea::TextArea;

const UNSAVED_FILE_NAME: &str = "unsaved";
const MAX_PATH_STRING_DEPTH: u16 = 10;

pub(super) struct Buffer<'a> {
    pub text_area: TextArea<'a>,
    pub file_path: Option<PathBuf>,
    pub modified: bool,
    pub clipboard_context: Option<ClipboardContext>,
    pub current_path_string: Option<String>,
}

impl Default for Buffer<'_> {
    fn default() -> Self {
        Self {
            text_area: Default::default(),
            file_path: Default::default(),
            modified: Default::default(),
            clipboard_context: new_clipboard(),
            current_path_string: Default::default(),
        }
    }
}

impl Buffer<'_> {
    pub(super) fn new(file: Option<PathBuf>) -> Self {
        let current_path_string = if let Some(path) = &file {
            Self::current_path(path, 10)
        } else {
            None
        };
        Self {
            file_path: file,
            current_path_string,
            ..Default::default()
        }
    }
    /// Gets the current file`s directory.
    /// For obtaining the file path, use `current_path`
    pub(super) fn current_directory(&self) -> PathBuf {
        self.file_path
            .clone()
            .map(|path| {
                if path.is_dir() {
                    path
                } else {
                    path.parent().unwrap_or(Path::new("/")).to_path_buf()
                }
            })
            .unwrap_or_else(|| current_dir().unwrap_or_default())
    }
    pub(super) fn clear_text(&mut self) {
        self.modified = false;
        self.text_area = TextArea::default();
    }
    pub(super) fn change_path(&mut self, path: PathBuf) {
        self.current_path_string = Self::current_path(&path, MAX_PATH_STRING_DEPTH);
        self.file_path = Some(path);
        self.modified = false;
    }
    pub fn file_name(&self) -> String {
        let Some(path) = &self.file_path else {
            return UNSAVED_FILE_NAME.to_string();
        };
        let Some(Some(file_name)) = path.file_name().map(|f| f.to_str()) else {
            return UNSAVED_FILE_NAME.to_string();
        };
        if path.exists() {
            file_name.to_string()
        } else {
            format!("{file_name} - {UNSAVED_FILE_NAME}")
        }
    }
    fn current_path(path: &Path, depth_limit: u16) -> Option<String> {
        let mut depth_limit = depth_limit;
        let mut p = path.parent();
        let mut vec: Vec<String> = Vec::new();
        while depth_limit > 0
            && let Some(dir) = p
            && let Some(file_name) = dir.file_name()
        {
            vec.push(file_name.to_string_lossy().to_string());
            depth_limit -= 1;
            p = dir.parent();
        }
        vec.reverse();
        let mapped = vec.join("/");
        let r = if depth_limit == 0 {
            format!(".../{}", mapped)
        } else {
            mapped
        };
        Some(r)
    }
    pub fn push_to_clipboard(&mut self, text: String) -> color_eyre::Result<()> {
        let Some(clipboard) = self.clipboard_context.as_mut() else {
            return Err(eyre!("Clipboard is unavailable"));
        };
        clipboard
            .set_contents(text)
            .map_err(|e| eyre!(e.to_string()))?;
        Ok(())
    }
    pub fn get_from_clipboard(&mut self) -> Option<String> {
        let copied = self.clipboard_context.as_mut()?;
        copied.get_contents().ok()
    }
}
