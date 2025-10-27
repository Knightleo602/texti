use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::widgets::{Block, BorderType};

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
