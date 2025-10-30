use crate::action::{
    Action, ActionResult, ActionSender, AsyncAction, AsyncActionSender, SaveFileResult,
    SelectorType,
};
use crate::component::component_utils::{center, default_block, write_file};
use crate::component::file_selector::component::FileSelectorComponent;
use crate::component::help::{HelpComponent, KEYBINDS_HELP_TITLE};
use crate::component::notification::NotificationComponent;
use crate::component::{AppComponent, Component};
use crate::config::Config;
use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::eyre::{eyre, Result};
use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::Frame;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use throbber_widgets_tui::{Throbber, BRAILLE_SIX_DOUBLE};
use tui_textarea::{CursorMove, TextArea};

const UNSAVED_FILE_NAME: &str = "unsaved";

struct Buffer<'a> {
    text_area: TextArea<'a>,
    file_path: Option<PathBuf>,
    modified: bool,
    clipboard_context: Option<ClipboardContext>,
}

fn new_clipboard() -> Option<ClipboardContext> {
    ClipboardContext::new().ok()
}

impl Default for Buffer<'_> {
    fn default() -> Self {
        Self {
            text_area: Default::default(),
            file_path: Default::default(),
            modified: Default::default(),
            clipboard_context: new_clipboard(),
        }
    }
}

impl Buffer<'_> {
    fn new(file: Option<PathBuf>) -> Self {
        Self {
            text_area: TextArea::default(),
            file_path: file,
            modified: false,
            clipboard_context: new_clipboard(),
        }
    }
    fn current_path(&self) -> PathBuf {
        self.file_path
            .clone()
            .unwrap_or_else(|| current_dir().unwrap_or_default())
    }
    fn clear(&mut self) {
        self.modified = false;
        self.text_area = TextArea::default();
    }
    pub fn file_name(&self) -> String {
        let Some(path) = &self.file_path else {
            return UNSAVED_FILE_NAME.to_string();
        };
        let Some(Some(file_name)) = path.file_name().map(|f| f.to_str()) else {
            return UNSAVED_FILE_NAME.to_string();
        };
        file_name.to_string()
    }
    pub fn file_path(&self) -> Option<String> {
        self.file_path
            .as_ref()?
            .parent()
            .map(|p| p.to_string_lossy().to_string())
    }
    pub fn push_to_clipboard(&mut self, text: String) -> Result<()> {
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

#[derive(Default)]
pub struct EditorComponent<'a> {
    buffer: Buffer<'a>,
    loading: bool,
    saving_file: bool,
    action_sender: Option<ActionSender>,
    task_result_sender: Option<AsyncActionSender>,
    insert: bool,
    config: Config,
    help_key: Option<char>,
    notification: NotificationComponent,
    help_component: Option<HelpComponent>,
    file_dialog: FileSelectorComponent<'a>,
}

impl<P: AsRef<Path>> From<P> for EditorComponent<'_> {
    fn from(value: P) -> Self {
        let path = PathBuf::from(value.as_ref());
        let buffer = Buffer::new(Some(path));
        Self {
            buffer,
            ..Default::default()
        }
    }
}

impl EditorComponent<'_> {
    pub fn new<S: AsRef<str>>(file: S) -> Self {
        let path = PathBuf::from(file.as_ref());
        let buffer = Buffer::new(Some(path));
        Self {
            buffer,
            ..Default::default()
        }
    }
    fn load_file(&mut self) {
        let Some(path) = &mut self.buffer.file_path else {
            return;
        };
        let action_sender = self.task_result_sender.clone().unwrap();
        let path = path.clone();
        self.loading = true;
        tokio::spawn(async move {
            if !path.exists() || path.is_dir() {
                let _ = action_sender.send(AsyncAction::LoadFileContents(String::new()));
                return;
            }
            let res = tokio::fs::read(path).await;
            match res {
                Ok(contents) => {
                    let string = String::from_utf8(contents).unwrap();
                    let action = AsyncAction::LoadFileContents(string);
                    let _ = action_sender.send(action);
                }
                Err(err) => {
                    let action = AsyncAction::Error(format!("{:?}", err));
                    let _ = action_sender.send(action);
                }
            }
        });
    }
    fn handle_selector(&mut self, path_buf: PathBuf, selector_type: SelectorType) -> ActionResult {
        match selector_type {
            SelectorType::PickFolder | SelectorType::NewFile => self.save_file_at(path_buf),
            SelectorType::PickFile => {
                self.buffer.file_path = Some(path_buf);
                self.load_file();
                ActionResult::consumed(true)
            }
        }
    }
    fn save_file(&mut self) -> ActionResult {
        if !self.buffer.modified && self.buffer.file_path.is_some() {
            return ActionResult::not_consumed(false);
        }
        let Some(path) = self.buffer.file_path.clone() else {
            return self.open_file_dialog(SelectorType::NewFile);
        };
        self.save_file_at(path)
    }
    fn save_file_at(&mut self, path: PathBuf) -> ActionResult {
        self.buffer.file_path = Some(path.clone());
        let lines = self.buffer.text_area.lines().join("\n");
        let action_sender = self.task_result_sender.clone().unwrap();
        self.saving_file = true;
        self.file_dialog.hide();
        tokio::spawn(async move {
            let r = write_file(path, lines).await;
            let _ = action_sender.send(r);
        });
        ActionResult::consumed(true)
    }
    fn show_help(&mut self) {
        let Some(comp) =
            HelpComponent::from_component(KEYBINDS_HELP_TITLE, AppComponent::Editor, &self.config)
        else {
            return;
        };
        self.help_component = Some(comp);
    }
    fn hide_help(&mut self) {
        self.help_component = None;
    }
    fn start_selection(&mut self) {
        if !self.buffer.text_area.is_selecting() {
            self.buffer.text_area.start_selection();
        }
    }
    fn stop_selection(&mut self) {
        self.buffer.text_area.cancel_selection();
    }
    fn move_cursor(&mut self, cursor_move: CursorMove) -> ActionResult {
        self.buffer.text_area.move_cursor(cursor_move);
        ActionResult::consumed(true)
    }
    fn delete(&mut self) -> ActionResult {
        if self.buffer.text_area.delete_char() {
            ActionResult::consumed(true)
        } else {
            ActionResult::not_consumed(false)
        }
    }
    fn cut_selection(&mut self) -> ActionResult {
        self.buffer.text_area.cut();
        let yanked = self.buffer.text_area.yank_text();
        self.stop_selection();
        if yanked.is_empty() {
            return ActionResult::Consumed { rerender: true };
        }
        match self.buffer.push_to_clipboard(yanked) {
            Ok(_) => self.notification.notify_text("Cut"),
            Err(e) => self.notification.notify_error(e),
        }
        ActionResult::consumed(true)
    }
    fn add_char(&mut self, char: char) -> ActionResult {
        if self.buffer.text_area.is_selecting() {
            let previous_yank = self.buffer.text_area.yank_text();
            self.buffer.text_area.cut();
            self.buffer.text_area.cancel_selection();
            self.buffer.text_area.set_yank_text(previous_yank)
        }
        self.buffer.text_area.insert_char(char);
        self.buffer.modified = true;
        ActionResult::consumed(true)
    }
    fn backspace(&mut self) -> ActionResult {
        self.buffer.text_area.delete_char();
        self.buffer.modified = true;
        ActionResult::consumed(true)
    }
    fn new_line(&mut self) -> ActionResult {
        self.buffer.text_area.insert_newline();
        self.buffer.modified = true;
        ActionResult::consumed(true)
    }
    fn tab(&mut self) -> ActionResult {
        self.buffer.text_area.insert_tab();
        self.buffer.modified = true;
        ActionResult::consumed(true)
    }
    fn toggle_help(&mut self) -> ActionResult {
        if self.help_component.is_some() {
            self.hide_help();
        } else {
            self.show_help();
        }
        ActionResult::consumed(true)
    }
    fn load_file_contents(&mut self, contents: String) -> ActionResult {
        self.loading = false;
        self.buffer.clear();
        self.buffer.text_area.insert_str(contents);
        self.buffer.text_area.cancel_selection();
        ActionResult::consumed(true)
    }
    fn begin_insert_mode(&mut self) -> ActionResult {
        self.insert = true;
        ActionResult::consumed(true)
    }
    fn copy_selection(&mut self) -> ActionResult {
        self.buffer.text_area.copy();
        let yanked = self.buffer.text_area.yank_text();
        if yanked.is_empty() {
            return ActionResult::consumed(false);
        }
        if let Err(e) = self.buffer.push_to_clipboard(yanked) {
            self.notification.notify_error(e)
        } else {
            self.notification.notify_text("Copied")
        }
        ActionResult::consumed(true)
    }
    fn paste_text_from_clipboard(&mut self) -> ActionResult {
        let Some(contents) = self.buffer.get_from_clipboard() else {
            return ActionResult::consumed(false);
        };
        self.paste_text(contents)
    }
    fn paste_text(&mut self, text: String) -> ActionResult {
        let changed = self.buffer.text_area.insert_str(text);
        ActionResult::consumed(changed)
    }
    fn select_all(&mut self) -> ActionResult {
        self.buffer.text_area.select_all();
        ActionResult::consumed(true)
    }
    fn handle_file_saved(&mut self, result: SaveFileResult) -> ActionResult {
        if self.saving_file {
            self.saving_file = false;
            match result {
                SaveFileResult::Saved(path) => {
                    self.notification.notify_text("File saved");
                    self.buffer.file_path = Some(path)
                }
                SaveFileResult::Error(error) => self.notification.notify_error(error),
                SaveFileResult::MissingName => return self.open_file_dialog(SelectorType::NewFile),
            };
            ActionResult::consumed(true)
        } else {
            Default::default()
        }
    }
    fn open_file_dialog(&mut self, selector_type: SelectorType) -> ActionResult {
        self.file_dialog
            .show(self.buffer.current_path(), selector_type);
        ActionResult::consumed(true)
    }

    fn page_up(&mut self) -> ActionResult {
        self.buffer.text_area.move_cursor(CursorMove::Top);
        ActionResult::consumed(true)
    }

    fn page_down(&mut self) -> ActionResult {
        self.buffer.text_area.move_cursor(CursorMove::Down);
        ActionResult::consumed(true)
    }

    fn move_next_word(&mut self) -> ActionResult {
        self.buffer.text_area.move_cursor(CursorMove::WordForward);
        ActionResult::consumed(true)
    }

    fn move_previous_word(&mut self) -> ActionResult {
        self.buffer.text_area.move_cursor(CursorMove::WordBack);
        ActionResult::consumed(true)
    }
}

impl Component for EditorComponent<'_> {
    fn register_config(&mut self, config: &Config) {
        if let Some(event) = config
            .keybindings
            .get_key_event_of_action(AppComponent::Editor, Action::Help)
        {
            self.help_key = event.code.as_char();
        }
        self.file_dialog.register_config(config);
        self.config = config.clone();
    }
    fn register_action_sender(&mut self, sender: ActionSender) {
        self.action_sender = Some(sender.clone());
        self.file_dialog.register_action_sender(sender);
    }
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.task_result_sender = Some(sender.clone());
        self.file_dialog.register_async_action_sender(sender)
    }
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        if let Some(a) = self.file_dialog.override_keybind_id(key_event) {
            return Some(a);
        };
        Some(&AppComponent::Editor)
    }
    fn handle_action(&mut self, action: Action) -> ActionResult {
        let notification_res = self.notification.handle_action_ref(&action);
        if notification_res.is_consumed() {
            return notification_res;
        }
        let file_dialog_res = self.file_dialog.handle_action(action.clone());
        if file_dialog_res.is_consumed() {
            return file_dialog_res;
        }
        match action {
            Action::Tick => return self.notification.handle_tick_action(),
            Action::Character(char) => return self.add_char(char),
            Action::Backspace => return self.backspace(),
            Action::NewLine => return self.new_line(),
            Action::Tab => return self.tab(),
            Action::Delete => return self.delete(),
            Action::Help => return self.toggle_help(),
            Action::Insert => return self.begin_insert_mode(),
            Action::Left => {
                self.stop_selection();
                return self.move_cursor(CursorMove::Back);
            }
            Action::SelectLeft => {
                self.start_selection();
                return self.move_cursor(CursorMove::Back);
            }
            Action::Right => {
                self.stop_selection();
                return self.move_cursor(CursorMove::Forward);
            }
            Action::SelectRight => {
                self.start_selection();
                return self.move_cursor(CursorMove::Forward);
            }
            Action::Up => {
                self.stop_selection();
                return self.move_cursor(CursorMove::Up);
            }
            Action::SelectUp => {
                self.start_selection();
                return self.move_cursor(CursorMove::Up);
            }
            Action::Down => {
                self.stop_selection();
                return self.move_cursor(CursorMove::Down);
            }
            Action::SelectDown => {
                self.start_selection();
                return self.move_cursor(CursorMove::Down);
            }
            Action::Cancel => {
                if self.buffer.text_area.is_selecting() {
                    self.buffer.text_area.cancel_selection();
                    return ActionResult::Consumed { rerender: true };
                }
                if self.insert {
                    self.insert = false;
                    return ActionResult::Consumed { rerender: true };
                }
            }
            Action::Copy => return self.copy_selection(),
            Action::Paste => return self.paste_text_from_clipboard(),
            Action::PasteText(text) => return self.paste_text(text),
            Action::Cut => return self.cut_selection(),
            Action::SelectAll => return self.select_all(),
            Action::Save => return self.save_file(),
            Action::Redo => {
                if self.buffer.text_area.redo() {
                    return ActionResult::consumed(true);
                }
            }
            Action::Undo => {
                if self.buffer.text_area.undo() {
                    return ActionResult::consumed(true);
                }
            }
            Action::Return => {
                let _ = self
                    .task_result_sender
                    .as_ref()
                    .unwrap()
                    .send(AsyncAction::Navigate(None));
            }
            Action::OpenFile => return self.open_file_dialog(SelectorType::PickFile),
            Action::PageUp => return self.page_up(),
            Action::PageDown => return self.page_down(),
            Action::EndOfWord => return self.move_next_word(),
            Action::StartOfWord => return self.move_previous_word(),
            _ => {}
        };
        Default::default()
    }
    fn handle_async_action(&mut self, action: AsyncAction) -> ActionResult {
        match action {
            AsyncAction::LoadFileContents(string) => return self.load_file_contents(string),
            AsyncAction::SavedFile(result) => return self.handle_file_saved(result),
            AsyncAction::Error(msg) => {
                self.notification.notify_error(msg);
                return ActionResult::consumed(true);
            }
            AsyncAction::SelectPath(path, selector) => return self.handle_selector(path, selector),
            _ => {}
        }
        Default::default()
    }
    fn init(&mut self) {
        self.load_file();
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let file_title = format!(" {} ", self.buffer.file_name());
        let file_title = Line::from(file_title).centered();
        let mut block = default_block().title_top(file_title);
        let mode_title = if self.insert { " Insert " } else { " Normal " };
        let help_title = format!(" [{}] Help ", self.help_key.unwrap_or(' '));
        let help_title = Line::from(help_title).right_aligned();
        let mode_title = Line::raw(mode_title).left_aligned();
        block = block.title_bottom(help_title);
        block = block.title_bottom(mode_title);
        if let Some(file_path) = self.buffer.file_path() {
            let file_path_title = format!(" {} ", file_path);
            let file_path_title = Line::from(file_path_title).left_aligned();
            block = block.title_top(file_path_title);
        }
        if self.buffer.modified {
            let modified_title = Line::raw(" Unsaved changes ").right_aligned();
            block = block.title_top(modified_title);
        }
        let block_area = if let Some(help_component) = &mut self.help_component {
            let [block_area, help_area] = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1), Constraint::Max(4)])
                .areas(area);
            help_component.render(frame, help_area);
            frame.render_widget(&block, block_area);
            block.inner(block_area)
        } else {
            frame.render_widget(&block, area);
            block.inner(area)
        };
        self.notification.render(frame, block_area);
        let [block_area] = Layout::default()
            .constraints([Constraint::Fill(1)])
            .areas(block_area);
        if self.loading {
            let area = center(block_area);
            let loader = Throbber::default().throbber_set(BRAILLE_SIX_DOUBLE);
            frame.render_widget(loader, area);
        } else {
            frame.render_widget(&self.buffer.text_area, block_area);
        }
        self.file_dialog.render(frame, area);
    }
}
