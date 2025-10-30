use crate::action::{ActionResult, SelectorType};
use crate::component::file_selector::create_default_text_area;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use tui_textarea::{CursorMove, TextArea};

const FILE_NAME_TITLE: &str = " File Name ";
const SEARCH_BOX_TITLE: &str = " Search Folder ";

#[derive(Default)]
pub(super) struct FileSelectorInput<'a> {
    text_area: Option<TextArea<'a>>,
    filter: Option<TextArea<'a>>,
    selector_type: SelectorType,
}

impl FileSelectorInput<'_> {
    pub fn change_type(&mut self, selector_type: SelectorType) {
        self.selector_type = selector_type;
        match self.selector_type {
            SelectorType::PickFolder | SelectorType::PickFile => {
                self.filter = Some(create_default_text_area(SEARCH_BOX_TITLE))
            }
            SelectorType::NewFile => {
                self.text_area = Some(create_default_text_area(FILE_NAME_TITLE))
            }
        }
    }
    pub fn selector_type(&self) -> SelectorType {
        self.selector_type
    }
    pub fn clear(&mut self) {
        self.filter = None;
        self.text_area = None;
    }
    pub fn current_filter(&self) -> Option<String> {
        if let Some(filter) = &self.filter
            && !filter.is_empty()
        {
            return Some(filter.lines()[0].to_string());
        };
        None
    }
    pub fn current_text_area(&self) -> Option<String> {
        if let Some(filter) = &self.text_area
            && !filter.is_empty()
        {
            return Some(filter.lines()[0].to_string());
        };
        None
    }
    pub fn delete(&mut self) -> ActionResult {
        if let Some(text_area) = self.text_area.as_mut() {
            return ActionResult::consumed(text_area.delete_next_char());
        } else if let Some(text_area) = self.filter.as_mut() {
            return ActionResult::consumed(text_area.delete_next_char());
        }
        ActionResult::default()
    }
    pub fn backspace(&mut self) -> bool {
        if let Some(text_area) = self.text_area.as_mut() {
            return text_area.delete_char();
        } else if let Some(text_area) = self.filter.as_mut() {
            return text_area.delete_char();
        }
        false
    }
    pub fn filter_active(&self) -> bool {
        self.filter.is_some()
    }
    pub fn handle_character(&mut self, character: char) -> bool {
        if let Some(text_area) = self.text_area.as_mut() {
            text_area.insert_char(character);
            return true;
        } else if let Some(text_area) = self.filter.as_mut() {
            text_area.insert_char(character);
            return false;
        }
        false
    }
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.selector_type == SelectorType::NewFile {
            self.render_new_file_inputs(frame, area);
        } else {
            self.render_open_dir_inputs(frame, area);
        };
    }
    fn render_new_file_inputs(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default().direction(Direction::Horizontal);
        let input_text_area = self.text_area.as_ref().unwrap();
        let input_area = if let Some(text_area) = &self.filter {
            let [input_area, filter_area] = layout
                .constraints([Constraint::Percentage(70), Constraint::Fill(1)])
                .areas(area);
            frame.render_widget(text_area, filter_area);
            input_area
        } else {
            area
        };
        frame.render_widget(input_text_area, input_area);
    }
    fn render_open_dir_inputs(&self, frame: &mut Frame, area: Rect) {
        if let Some(text_area) = &self.filter {
            frame.render_widget(text_area, area);
        }
    }
    pub fn move_cursor(&mut self, cursor_move: CursorMove) {
        if let Some(text_area) = self.filter.as_mut() {
            text_area.move_cursor(cursor_move);
        } else if let Some(text_area) = self.text_area.as_mut() {
            text_area.move_cursor(cursor_move);
        }
    }
    pub fn toggle_filter(&mut self) -> ActionResult {
        if self.selector_type == SelectorType::PickFolder {
            return ActionResult::not_consumed(false);
        }
        self.filter = Some(create_default_text_area(SEARCH_BOX_TITLE));
        ActionResult::consumed(true)
    }
    pub fn cancel(&mut self) -> bool {
        if self.selector_type == SelectorType::NewFile && self.filter.take().is_some() {
            return true;
        }
        false
    }
}
