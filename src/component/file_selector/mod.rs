use crate::component::component_utils::default_block;
use ratatui::prelude::Line;
use std::path::PathBuf;
use tui_textarea::TextArea;

pub mod component;
mod input;
mod preview_component;

pub(super) fn create_default_text_area(title: &'_ str) -> TextArea<'_> {
    let title = Line::raw(title).left_aligned();
    let block = default_block().title_top(title);
    let mut text_area = TextArea::default();
    text_area.set_block(block);
    text_area
}

#[derive(Clone, Debug)]
pub(super) enum PathChild {
    File {
        full_file_name: String,
        icon: Option<String>,
    },
    Folder(String),
    MoveUp,
}

impl PathChild {
    pub fn file(file_name: String, path_buf: PathBuf) -> Self {
        let extension = path_buf
            .extension()
            .unwrap_or_default()
            .display()
            .to_string();
        let icon = icon_for_file(&file_name, &extension);
        Self::File {
            full_file_name: file_name,
            icon,
        }
    }
    fn filter<F: AsRef<str>>(&self, filter: F) -> bool {
        let filter = filter.as_ref();
        match self {
            PathChild::File {
                full_file_name,
                icon: _,
            } => full_file_name.to_lowercase().contains(filter),
            PathChild::Folder(f) => f.to_lowercase().contains(filter),
            PathChild::MoveUp => true,
        }
    }

    fn to_path_line(&self) -> String {
        match self {
            PathChild::File {
                full_file_name,
                icon,
            } => {
                if let Some(icon) = icon {
                    format!("{icon} {full_file_name}")
                } else {
                    full_file_name.to_string()
                }
            }
            PathChild::Folder(path) => format!(" {}", path),
            PathChild::MoveUp => "...".to_string(),
        }
    }
}

fn icon_for_file(file_name: &str, ext: &str) -> Option<String> {
    let r = match ext {
        "rs" => "",
        "txt" => "󰦨",
        "yaml" => "",
        "json" | "json5" => "",
        "toml" => "",
        "java" => "",
        "js" => "",
        "ts" => "",
        "kt" => "",
        "c" => "",
        "cpp" => "",
        "cs" => "",
        "css" => "",
        "html" => "",
        _ => match file_name {
            ".config" => "",
            _ => return None,
        },
    };
    Some(r.to_string())
}
