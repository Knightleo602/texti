use crate::action::SaveFileResult;
use clipboard::{ClipboardContext, ClipboardProvider};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Block, BorderType};
use std::fs;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub(super) fn center_horizontally(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::horizontal([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub(super) fn center_vertically(area: Rect, constraint: Constraint) -> Rect {
    let [area] = Layout::vertical([constraint])
        .flex(Flex::Center)
        .areas(area);
    area
}

pub(super) fn center(area: Rect) -> Rect {
    let area = center_horizontally(area, Constraint::Percentage(50));
    center_vertically(area, Constraint::Percentage(50))
}

pub(super) fn default_block() -> Block<'static> {
    Block::bordered().border_type(BorderType::Rounded)
}

pub(super) async fn write_file(path: PathBuf, lines: String, overwrite: bool) -> SaveFileResult {
    let exists = path.exists();
    if exists {
        if !overwrite {
            return SaveFileResult::ConfirmOverwrite;
        }
    } else if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        let result = SaveFileResult::Error(e.to_string());
        return result;
    }
    if path.is_dir() {
        return SaveFileResult::MissingName;
    }
    let mut file = match File::create(&path).await {
        Ok(file) => file,
        Err(e) => {
            let result = SaveFileResult::Error(e.to_string());
            return result;
        }
    };
    let result = if let Err(e) = file.write_all(lines.as_ref()).await {
        SaveFileResult::Error(e.to_string())
    } else {
        SaveFileResult::Saved(path)
    };
    if let Err(e) = file.flush().await {
        return SaveFileResult::Error(e.to_string());
    }
    result
}

pub(super) fn new_clipboard() -> Option<ClipboardContext> {
    ClipboardContext::new().ok()
}

#[inline]
pub(super) fn key_label_format(key: &str, label: &str) -> String {
    format!(" [{key}] {label} ")
}
