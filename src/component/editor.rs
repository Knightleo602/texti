use crate::action::{Action, ActionResult, ActionSender, SaveFileResult};
use crate::component::component_utils::{center, default_block};
use crate::component::help::{HelpComponent, KEYBINDS_HELP_TITLE};
use crate::component::notification::NotificationComponent;
use crate::component::{AppComponent, Component};
use crate::config::Config;
use clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::eyre::{eyre, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::Frame;
use std::fs;
use std::path::PathBuf;
use throbber_widgets_tui::{Throbber, BRAILLE_SIX_DOUBLE};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
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
}

#[derive(Default)]
pub struct EditorComponent<'a> {
    buffer: Buffer<'a>,
    loading: bool,
    saving_file: bool,
    action_sender: Option<ActionSender>,
    insert: bool,
    config: Config,
    help_key: Option<char>,
    creating_file_name: bool,
    notification: NotificationComponent,
    help_component: Option<HelpComponent>,
}

impl EditorComponent<'_> {
    pub fn new(file: String) -> Self {
        let path = PathBuf::from(file);
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
        let action_sender = self.action_sender.clone().unwrap();
        let path = path.clone();
        self.loading = true;
        tokio::spawn(async move {
            if !path.exists() || path.is_dir() {
                let _ = action_sender.send(Action::LoadFileContents(String::new()));
                return;
            }
            let res = tokio::fs::read(path).await;
            match res {
                Ok(contents) => {
                    let string = String::from_utf8(contents).unwrap();
                    let action = Action::LoadFileContents(string);
                    let _ = action_sender.send(action);
                }
                Err(err) => {
                    let action = Action::Error(format!("{:?}", err));
                    let _ = action_sender.send(action);
                }
            }
        });
    }
    fn save_file(&mut self) -> bool {
        if !self.buffer.modified {
            return false;
        }
        let Some(path) = self.buffer.file_path.clone() else {
            return false;
        };
        let lines = self.buffer.text_area.lines().join("\n");
        let action_sender = self.action_sender.clone().unwrap();
        self.saving_file = true;
        tokio::spawn(async move {
            if !path.exists()
                && let Some(parent) = path.parent()
                && let Err(e) = fs::create_dir_all(parent)
            {
                let result = SaveFileResult::Error(e.to_string());
                let _ = action_sender.send(Action::SavedFile(result));
                return;
            };
            let mut file = match File::create(&path).await {
                Ok(file) => file,
                Err(e) => {
                    let result = SaveFileResult::Error(e.to_string());
                    let _ = action_sender.send(Action::SavedFile(result));
                    return;
                }
            };
            let result = if let Err(e) = file.write_all(lines.as_ref()).await {
                SaveFileResult::Error(e.to_string())
            } else {
                SaveFileResult::Saved(path)
            };
            if let Err(e) = file.flush().await {
                let result = SaveFileResult::Error(e.to_string());
                let _ = action_sender.send(Action::SavedFile(result));
                return;
            }
            let _ = action_sender.send(Action::SavedFile(result));
        });
        true
    }
    fn handle_ctrl_key_event(&mut self, key_event: KeyEvent) -> bool {
        if !key_event.modifiers.contains(KeyModifiers::CONTROL) {
            return false;
        };
        let Some(action) = self.get_action(key_event) else {
            return true;
        };
        let _ = self.action_sender.as_ref().unwrap().send(action);
        true
    }
    fn handle_shift_key_event(&mut self, key_event: KeyEvent) -> bool {
        if !key_event.modifiers.contains(KeyModifiers::SHIFT) {
            if let Some(action) = self.get_action(key_event)
                && action.is_directional_action()
            {
                self.buffer.text_area.cancel_selection();
            };
            return false;
        };
        let Some(action) = self.get_action(key_event) else {
            return true;
        };
        if action.is_directional_action() {
            if !self.buffer.text_area.is_selecting() {
                self.buffer.text_area.start_selection();
            }
        } else {
            self.buffer.text_area.cancel_selection()
        }
        let _ = self.action_sender.as_ref().unwrap().send(action);
        true
    }
    fn get_action(&mut self, key_event: KeyEvent) -> Option<Action> {
        self.config
            .keybindings
            .get_action(AppComponent::Editor, key_event)
    }
    fn handle_normal_key_event(&mut self, key_event: KeyEvent) -> ActionResult {
        match key_event.code {
            KeyCode::Char(char) => {
                if self.buffer.text_area.is_selecting() {
                    let previous_yank = self.buffer.text_area.yank_text();
                    self.buffer.text_area.cut();
                    self.buffer.text_area.cancel_selection();
                    self.buffer.text_area.set_yank_text(previous_yank)
                }
                self.buffer.text_area.insert_char(char);
                self.buffer.modified = true;
                return ActionResult::consumed(true);
            }
            KeyCode::Backspace => {
                self.buffer.text_area.delete_char();
                self.buffer.modified = true;
                return ActionResult::consumed(true);
            }
            KeyCode::Enter => {
                self.buffer.text_area.insert_newline();
                self.buffer.modified = true;
                return ActionResult::consumed(true);
            }
            KeyCode::Tab => {
                self.buffer.text_area.insert_tab();
                self.buffer.modified = true;
                return ActionResult::consumed(true);
            }
            KeyCode::Delete => {
                self.buffer.text_area.delete_next_char();
                self.buffer.modified = true;
                return ActionResult::consumed(true);
            }
            _ => {}
        };
        Default::default()
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
}

impl<'a> Component for EditorComponent<'a> {
    fn register_config(&mut self, config: &Config) {
        if let Some(event) = config
            .keybindings
            .get_key_event_of_action(AppComponent::Editor, Action::Help)
        {
            self.help_key = event.code.as_char();
        }
        self.config = config.clone();
    }
    fn set_action_sender(&mut self, sender: ActionSender) {
        self.action_sender = Some(sender);
    }
    fn handle_action(&mut self, action: Action) -> ActionResult {
        let notification_res = self.notification.handle_action_ref(&action);
        if notification_res.is_consumed() {
            return notification_res;
        }
        match action {
            Action::Tick => {
                return self.notification.handle_tick_action();
            }
            Action::Help => {
                if self.help_component.is_some() {
                    self.hide_help();
                } else {
                    self.show_help();
                }
                return ActionResult::consumed(true);
            }
            Action::LoadFileContents(string) => {
                self.loading = false;
                self.buffer.clear();
                self.buffer.text_area.insert_str(string);
                self.buffer.text_area.cancel_selection();
                return ActionResult::Consumed { rerender: true };
            }
            Action::Insert => {
                self.insert = true;
                return ActionResult::Consumed { rerender: true };
            }
            Action::Left => {
                self.buffer.text_area.move_cursor(CursorMove::Back);
                return ActionResult::Consumed { rerender: true };
            }
            Action::Right => {
                self.buffer.text_area.move_cursor(CursorMove::Forward);
                return ActionResult::Consumed { rerender: true };
            }
            Action::Up => {
                self.buffer.text_area.move_cursor(CursorMove::Up);
                return ActionResult::Consumed { rerender: true };
            }
            Action::Down => {
                self.buffer.text_area.move_cursor(CursorMove::Down);
                return ActionResult::Consumed { rerender: true };
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
            Action::Copy => {
                self.buffer.text_area.copy();
                let yanked = self.buffer.text_area.yank_text();
                if yanked.is_empty() {
                    return ActionResult::Consumed { rerender: true };
                }
                match self.buffer.push_to_clipboard(yanked) {
                    Ok(_) => self.notification.notify_text("Copied"),
                    Err(e) => self.notification.notify_error(e),
                }
                return ActionResult::Consumed { rerender: true };
            }
            Action::Cut => {
                self.buffer.text_area.cut();
                let yanked = self.buffer.text_area.yank_text();
                self.buffer.text_area.cancel_selection();
                if yanked.is_empty() {
                    return ActionResult::Consumed { rerender: true };
                }
                match self.buffer.push_to_clipboard(yanked) {
                    Ok(_) => self.notification.notify_text("Cut"),
                    Err(e) => self.notification.notify_error(e),
                }
                return ActionResult::consumed(true);
            }
            Action::SelectAll => {
                self.buffer.text_area.select_all();
                return ActionResult::consumed(true);
            }
            Action::Save => {
                if self.save_file() {
                    return ActionResult::consumed(true);
                }
            }
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
            Action::SavedFile(result) => {
                if self.saving_file {
                    self.saving_file = false;
                    match result {
                        SaveFileResult::Saved(path) => {
                            self.notification.notify_text("File saved");
                            self.buffer.file_path = Some(path)
                        }
                        SaveFileResult::Error(error) => {
                            self.notification.notify_error(error);
                        }
                        SaveFileResult::MissingName => self.creating_file_name = true,
                    }
                    return ActionResult::consumed(true);
                }
            }
            Action::Return => self
                .action_sender
                .as_ref()
                .unwrap()
                .send(Action::Navigate(None))
                .unwrap(),
            Action::Error(msg) => self.notification.notify_error(msg),
            _ => {}
        };
        Default::default()
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ActionResult {
        if self.handle_shift_key_event(key_event) || self.handle_ctrl_key_event(key_event) {
            return ActionResult::Consumed { rerender: true };
        }
        if !self.insert {
            if self
                .config
                .keybindings
                .get_action(AppComponent::Editor, key_event)
                .is_none()
            {
                let r = self.handle_normal_key_event(key_event);
                if r.is_consumed() {
                    self.insert = true;
                }
                return r;
            }
            return Default::default();
        };
        self.handle_normal_key_event(key_event)
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
        block = block.title_bottom(help_title);
        let mode_title = Line::raw(mode_title).left_aligned();
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
    }
}
