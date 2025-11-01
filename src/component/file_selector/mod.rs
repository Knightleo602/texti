use crate::component::component_utils::default_block;
use ratatui::prelude::Line;
use ratatui::text::Text;
use tui_textarea::TextArea;

pub mod component;
mod input;

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
        extension: String,
    },
    Folder(String),
    MoveUp,
}

impl PathChild {
    fn filter<F: AsRef<str>>(&self, filter: F) -> bool {
        let filter = filter.as_ref();
        match self {
            PathChild::File {
                full_file_name,
                extension: _,
            } => full_file_name.to_lowercase().contains(filter),
            PathChild::Folder(f) => f.to_lowercase().contains(filter),
            PathChild::MoveUp => true,
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
        _ => {
            if file_name == ".config" {
                ""
            } else {
                return None;
            }
        }
    };
    Some(r.to_string())
}

impl From<&PathChild> for Text<'_> {
    fn from(value: &PathChild) -> Self {
        match value {
            PathChild::File {
                full_file_name,
                extension,
            } => {
                let t = if let Some(icon) = icon_for_file(full_file_name, extension) {
                    icon + " " + full_file_name
                } else {
                    full_file_name.to_string()
                };
                Text::from(t)
            }
            PathChild::Folder(path) => Text::from(format!("\u{ea83} {}", path)),
            PathChild::MoveUp => Text::raw("..."),
        }
    }
}
