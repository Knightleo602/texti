use crate::action::{Action, ActionResult, AsyncActionSender};
use crate::component::component_utils::default_block;
use crate::component::Component;
use crate::config::effects::floating_component_enter_effect;
use crate::config::effects_config::EffectRunner;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear};
use ratatui::Frame;
use tui_textarea::{CursorMove, TextArea};

#[derive(Debug, Default)]
pub(super) struct SearchBoxComponent<'a> {
    text_area: Option<TextArea<'a>>,
    error: bool,
    regex: bool,
    effect_runner: EffectRunner,
}

impl<'a> SearchBoxComponent<'a> {
    pub fn toggle(&mut self) {
        if self.text_area.is_some() {
            self.text_area = None;
        } else {
            let text_area = TextArea::default();
            self.text_area = Some(text_area);
            self.effect_runner
                .add_effect(floating_component_enter_effect());
            self.update_text_area_placeholder();
        }
    }
    pub fn visible(&self) -> bool {
        self.text_area.is_some()
    }
    pub fn is_error(&self) -> bool {
        self.error
    }
    pub fn apply_search_pattern(&mut self, search_area: &mut TextArea) {
        let Some(search_text_area) = &mut self.text_area else {
            let _ = search_area.set_search_pattern("");
            return;
        };
        let search = &search_text_area.lines()[0];
        let search = if self.regex {
            search
        } else {
            &regex::escape(search)
        };
        let r = search_area.set_search_pattern(search);
        self.error = r.is_err();
        if self.error {
            search_text_area.set_block(Self::search_block().style(Color::Red))
        } else {
            search_text_area.set_block(Self::search_block())
        }
    }
    pub fn stop_search(&mut self) {
        self.error = false;
        self.text_area = None;
        self.regex = false;
    }
    fn update_text_area_placeholder(&mut self) {
        let text_area = self.text_area.as_mut().unwrap();
        let placeholder = if self.regex { "Regex" } else { "Text" };
        text_area.set_placeholder_text(placeholder);
        text_area.set_placeholder_style(Style::new().fg(Color::DarkGray));
        text_area.set_block(Self::search_block());
    }
    fn search_block() -> Block<'static> {
        const TITLE: &str = " Search ";
        let line = Line::raw(TITLE).left_aligned();
        let actions_title = Line::raw("   select ").right_aligned();
        default_block().title_top(line).title_bottom(actions_title)
    }
    fn start_selection(&mut self) -> ActionResult {
        let text_area = self.text_area.as_mut().unwrap();
        if !text_area.is_selecting() {
            text_area.start_selection();
            ActionResult::consumed(true)
        } else {
            Default::default()
        }
    }
    fn stop_selection(&mut self) -> ActionResult {
        let text_area = self.text_area.as_mut().unwrap();
        if text_area.is_selecting() {
            text_area.cancel_selection();
            ActionResult::consumed(true)
        } else {
            Default::default()
        }
    }
    fn next_result(&mut self, text_area: &mut TextArea) -> ActionResult {
        let found = text_area.search_forward(false);
        ActionResult::consumed(found)
    }
    fn previous_result(&mut self, text_area: &mut TextArea) -> ActionResult {
        let found = text_area.search_back(false);
        ActionResult::consumed(found)
    }
    fn move_cursor(&mut self, cursor_move: CursorMove) -> ActionResult {
        let text_area = self.text_area.as_mut().unwrap();
        text_area.move_cursor(cursor_move);
        ActionResult::consumed(true)
    }
    fn handle_char(&mut self, c: char) -> ActionResult {
        let text_area = self.text_area.as_mut().unwrap();
        text_area.insert_char(c);
        ActionResult::consumed(true)
    }
    fn handle_delete(&mut self) -> ActionResult {
        let text_area = self.text_area.as_mut().unwrap();
        let deleted = text_area.delete_next_char();
        ActionResult::consumed(deleted)
    }
    fn handle_backspace(&mut self) -> ActionResult {
        let text_area = self.text_area.as_mut().unwrap();
        let deleted = text_area.delete_char();
        ActionResult::consumed(deleted)
    }
    fn receive_action(&mut self, action: &Action) -> (ActionResult, bool) {
        match action {
            Action::SelectRight => {
                self.start_selection();
                return (self.move_cursor(CursorMove::Forward), false);
            }
            Action::SelectLeft => {
                self.start_selection();
                return (self.move_cursor(CursorMove::Back), false);
            }
            Action::Left => {
                self.stop_selection();
                return (self.move_cursor(CursorMove::Back), false);
            }
            Action::Right => {
                self.stop_selection();
                return (self.move_cursor(CursorMove::Forward), false);
            }
            Action::EndOfWord => return (self.move_cursor(CursorMove::WordEnd), false),
            Action::StartOfWord => return (self.move_cursor(CursorMove::WordBack), false),
            Action::Character(char) => return (self.handle_char(*char), true),
            Action::Delete => return (self.handle_delete(), true),
            Action::Backspace => return (self.handle_backspace(), true),
            Action::Search => {
                self.toggle();
                return (ActionResult::consumed(true), true);
            }
            Action::ToggleSearchRegex => {
                self.regex = !self.regex;
                self.update_text_area_placeholder();
                return (ActionResult::consumed(true), true);
            }
            Action::Cancel => {
                self.stop_search();
                return (ActionResult::consumed(true), true);
            }
            _ => {}
        }
        (ActionResult::consumed(false), false)
    }
    pub fn handle_action(&mut self, action: &Action, text_area: &mut TextArea) -> ActionResult {
        if !self.visible() {
            return ActionResult::not_consumed(false);
        }
        let (res, update_search) = match action {
            Action::Down => return self.next_result(text_area),
            Action::Up => return self.previous_result(text_area),
            _ => self.receive_action(action),
        };
        if res.is_consumed() && update_search {
            self.apply_search_pattern(text_area);
        }
        res
    }
}

impl Component for SearchBoxComponent<'_> {
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.effect_runner.register_async_sender(sender)
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        if !self.visible() {
            return ActionResult::not_consumed(false);
        }
        self.receive_action(action).0
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(text_area) = &self.text_area {
            let [area] = Layout::default()
                .direction(Direction::Horizontal)
                .flex(Flex::End)
                .margin(1)
                .constraints([Constraint::Percentage(30)])
                .areas(area);
            let [area] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3)])
                .areas(area);
            frame.render_widget(Clear, area);
            frame.render_widget(text_area, area);
            self.effect_runner.process(frame.buffer_mut(), area);
        }
    }
}
