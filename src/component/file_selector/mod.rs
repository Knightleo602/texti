use crate::component::component_utils::default_block;
use ratatui::prelude::Line;
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
    File(String),
    Folder(String),
    MoveUp,
}

impl PathChild {
    fn filter<F: AsRef<str>>(&self, filter: F) -> bool {
        let filter = filter.as_ref();
        match self {
            PathChild::File(f) => f.contains(filter),
            PathChild::Folder(f) => f.contains(filter),
            PathChild::MoveUp => true,
        }
    }
}
