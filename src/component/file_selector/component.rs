pub(crate) use crate::action::{
    Action, ActionResult, AsyncAction, AsyncActionSender, SelectorType,
};
use crate::component::component_utils::{center_horizontally, center_vertically, default_block};
use crate::component::file_selector::input::FileSelectorInput;
use crate::component::file_selector::PathChild;
use crate::component::{AppComponent, Component};
use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Clear, List, ListDirection};
use ratatui::Frame;
use std::path::{Path, PathBuf};
use tui_textarea::CursorMove;

/// A file selector component. Shows a list of all contents inside `current_path` and allows the
/// user to change directories or select files through it.
#[derive(Default)]
pub struct FileSelectorComponent<'a> {
    action_sender: Option<AsyncActionSender>,
    current_path: PathBuf,
    children: Vec<PathChild>,
    filtered_paths: Option<Vec<PathChild>>,
    input: FileSelectorInput<'a>,
    visible: bool,
    selected_index: Option<usize>,
}

impl FileSelectorComponent<'_> {
    pub fn show<P: AsRef<Path>>(&mut self, dir: P, selector_type: SelectorType) {
        self.input.change_type(selector_type);
        self.visible = true;
        self.select_dir(dir)
    }
    pub fn select_dir<P: AsRef<Path>>(&mut self, dir: P) {
        let dir_path = dir.as_ref();
        let Ok(read_dir) = dir_path.read_dir() else {
            return;
        };
        self.children.clear();
        self.children.push(PathChild::MoveUp);
        self.current_path = dir_path.to_path_buf();
        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let c = if path.is_dir() {
                PathChild::Folder(name)
            } else if self.input.selector_type().show_files() {
                PathChild::File(name)
            } else {
                continue;
            };
            self.children.push(c);
        }
        self.refresh_filtered_items();
    }
    pub fn hide(&mut self) {
        self.visible = false;
        self.input.clear();
    }
    fn select(&mut self, folder: bool) -> ActionResult {
        let Some(index) = self.selected_index.take() else {
            let path = if let Some(text_area) = self.input.current_text_area() {
                self.current_path.join(text_area)
            } else {
                self.current_path.clone()
            };
            let sender = self.action_sender.as_ref().unwrap();
            let _ = sender.send(AsyncAction::SelectPath(path, self.input.selector_type()));
            return ActionResult::consumed(true);
        };
        let Some(child) = self.children.get(index) else {
            return ActionResult::consumed(false);
        };
        let can_pick_folder = folder && self.input.selector_type().can_pick_folder();
        let path = match child {
            PathChild::File(f) => self.current_path.join(f),
            PathChild::Folder(f) => {
                let path = self.current_path.join(f);
                if !can_pick_folder {
                    self.select_dir(path);
                    return ActionResult::consumed(true);
                }
                path
            }
            PathChild::MoveUp => {
                let path = self.current_path.parent().unwrap().to_path_buf();
                self.select_dir(&path);
                return ActionResult::consumed(true);
            }
        };
        self.hide();
        let sender = self.action_sender.as_ref().unwrap();
        let _ = sender.send(AsyncAction::SelectPath(path, self.input.selector_type()));
        ActionResult::consumed(true)
    }
    fn refresh_filtered_items(&mut self) -> ActionResult {
        self.filtered_paths = None;
        let Some(filter) = self.input.current_filter() else {
            return ActionResult::consumed(false);
        };
        self.filtered_paths = Some(
            self.children
                .iter()
                .filter(move |x| x.filter(&filter))
                .cloned()
                .collect(),
        );
        ActionResult::consumed(true)
    }
    fn handle_character(&mut self, character: char) -> ActionResult {
        if self.input.handle_character(character) {
            self.refresh_filtered_items();
            return ActionResult::consumed(true);
        }
        Default::default()
    }
    fn handle_backspace(&mut self) -> ActionResult {
        if self.input.backspace() {
            return ActionResult::consumed(true);
        }
        Default::default()
    }
    fn list_len(&self) -> usize {
        self.filtered_paths.as_ref().unwrap_or(&self.children).len()
    }
    fn move_cursor_right(&mut self) -> ActionResult {
        self.input.move_cursor(CursorMove::Forward);
        ActionResult::consumed(true)
    }
    fn move_cursor_left(&mut self) -> ActionResult {
        self.input.move_cursor(CursorMove::Back);
        ActionResult::consumed(true)
    }
    fn move_cursor_up(&mut self) -> ActionResult {
        if let Some(index) = self.selected_index.as_mut() {
            if *index == 0 {
                return ActionResult::consumed(false);
            };
            *index -= 1;
            return ActionResult::consumed(true);
        } else {
            let len = self.list_len();
            if len > 0 {
                self.selected_index = Some(len - 1);
                return ActionResult::consumed(true);
            }
        }
        ActionResult::default()
    }
    fn move_cursor_down(&mut self) -> ActionResult {
        let len = self.list_len();
        if let Some(index) = self.selected_index.as_mut() {
            if *index == len - 1 {
                return ActionResult::consumed(false);
            };
            *index += 1;
            return ActionResult::consumed(true);
        } else if len > 0 {
            self.selected_index = Some(0);
            return ActionResult::consumed(true);
        }
        ActionResult::default()
    }
    fn handle_cancel(&mut self) -> ActionResult {
        if self.input.cancel() {
            self.refresh_filtered_items();
            return ActionResult::consumed(true);
        }
        if self.selected_index.take().is_some() {
            return ActionResult::consumed(true);
        }
        self.visible = false;
        ActionResult::consumed(true)
    }
}

impl Component for FileSelectorComponent<'_> {
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.action_sender = Some(sender)
    }
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        let _ = key_event;
        if self.visible {
            Some(&AppComponent::FileDialog)
        } else {
            None
        }
    }
    fn handle_action(&mut self, action: Action) -> ActionResult {
        if !self.visible {
            return Default::default();
        }
        match action {
            Action::Up => return self.move_cursor_up(),
            Action::Down => return self.move_cursor_down(),
            Action::Left => return self.move_cursor_left(),
            Action::Right => return self.move_cursor_right(),
            Action::Confirm => return self.select(false),
            Action::Select => return self.select(true),
            Action::Cancel => return self.handle_cancel(),
            Action::Backspace => return self.handle_backspace(),
            Action::Search => return self.input.toggle_filter(),
            Action::Delete => return self.input.delete(),
            Action::Character(char) => return self.handle_character(char),
            _ => {}
        }
        Default::default()
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.visible {
            let path_title = format!(" {} ", self.current_path.to_str().unwrap_or("/"));
            let path_line = Line::from(path_title).left_aligned();
            let block = default_block().title_bottom(path_line);
            let children = self.filtered_paths.as_ref().unwrap_or(&self.children);
            let items = children.iter().enumerate().map(|(i, v)| {
                let mut item = match v {
                    PathChild::File(name) => Text::raw(name),
                    PathChild::Folder(path) => Text::raw(path),
                    PathChild::MoveUp => Text::raw("..."),
                };
                if self
                    .selected_index
                    .is_some_and(|selected_index| selected_index == i)
                {
                    item = item.bg(Color::White).fg(Color::Black);
                }
                item.alignment(Alignment::Center)
            });
            let list = List::new(items)
                .direction(ListDirection::TopToBottom)
                .block(block);
            let area = center_horizontally(area, Constraint::Percentage(70));
            let area = center_vertically(area, Constraint::Percentage(70));
            frame.render_widget(Clear, area);
            let [input_area, list_area] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Fill(1)])
                .areas(area);
            self.input.render(frame, input_area);
            frame.render_widget(list, list_area);
        }
    }
}
