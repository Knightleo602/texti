use crate::action::{Action, ActionResult, AsyncAction, AsyncActionSender, SelectorType};
use crate::component::component_utils::{
    center, center_horizontally, center_vertically, default_block, key_label_format,
};
use crate::component::effect_runner::EffectRunner;
use crate::component::file_selector::{label_for_file, render_preview_if_able, HIGHLIGHT_SYMBOL};
use crate::component::preview_component::PreviewComponent;
use crate::component::{AppComponent, Component};
use crate::config::effects::dialog_enter;
use crate::config::Config;
use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::Color;
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Clear, HighlightSpacing, List, ListDirection, ListItem, ListState};
use ratatui::Frame;
use std::fs::read_to_string;
use std::path::PathBuf;

pub(super) const HISTORY_FILE_NAME: &str = "file_history.txt";

#[derive(Default)]
struct FileHistoryKeybinds {
    up: String,
    down: String,
    confirm: String,
    cancel: String,
}

impl FileHistoryKeybinds {
    fn register_keybinds(&mut self, app_component: &AppComponent, config: &Config) {
        let keybinds = &config.keybindings;
        self.up = keybinds.get_key_string_or_default(Action::Up, app_component);
        self.down = keybinds.get_key_string_or_default(Action::Down, app_component);
        self.confirm = keybinds.get_key_string_or_default(Action::Confirm, app_component);
        self.cancel = keybinds.get_key_string_or_default(Action::Cancel, app_component);
    }
}

struct FileHistory {
    label: String,
    path: PathBuf,
    parent_label: Option<String>,
}

#[derive(Default)]
pub struct FileHistoryComponent {
    opened: bool,
    data_dir: PathBuf,
    files: Vec<FileHistory>,
    preview_component: PreviewComponent,
    list_state: ListState,
    async_action_sender: Option<AsyncActionSender>,
    effect_runner: EffectRunner,
    keybinds: FileHistoryKeybinds,
}

impl FileHistoryComponent {
    pub fn show(&mut self) -> Result<()> {
        self.opened = true;
        self.effect_runner
            .add_effect(dialog_enter(Color::from_u32(0x1d2021)));
        self.load_files()?;
        Ok(())
    }
    pub fn hide(&mut self) {
        self.files.clear();
        self.opened = false;
    }
    pub fn showing(&self) -> bool {
        self.opened
    }
    pub fn reload_history(&mut self) -> Result<()> {
        self.files.clear();
        self.load_files()
    }
    fn load_files(&mut self) -> Result<()> {
        if !self.files.is_empty() {
            return Ok(());
        }
        let file = self.data_dir.join(HISTORY_FILE_NAME);
        let file = read_to_string(&file)?;
        for line in file.lines() {
            let path = PathBuf::from(line);
            if path.is_file() {
                let full_path = path.parent().map(|p| p.display().to_string());
                let label = label_for_file(&path);
                let file = FileHistory {
                    label,
                    path,
                    parent_label: full_path,
                };
                self.files.push(file);
            }
        }
        Ok(())
    }
    fn update_preview(&mut self, index: usize) {
        let path = &self.files[index].path;
        self.preview_component.change_dir(Some(path.clone()));
    }
    fn move_down(&mut self) -> ActionResult {
        if self.files.is_empty() {
            return ActionResult::consumed(false);
        }
        if let Some(selected) = self.list_state.selected() {
            if selected == self.files.len() - 1 {
                return ActionResult::consumed(false);
            }
            let i = selected + 1;
            self.update_preview(i);
            self.list_state.select(Some(i));
            return ActionResult::consumed(true);
        }
        self.update_preview(0);
        self.list_state.select(Some(0));
        ActionResult::consumed(true)
    }
    fn move_up(&mut self) -> ActionResult {
        if self.files.is_empty() {
            return ActionResult::consumed(false);
        }
        if let Some(selected) = self.list_state.selected() {
            if selected == 0 {
                return ActionResult::consumed(false);
            }
            let i = selected - 1;
            self.update_preview(i);
            self.list_state.select(Some(i));
            return ActionResult::consumed(true);
        }
        let last_index = self.files.len() - 1;
        self.update_preview(last_index);
        self.list_state.select(Some(last_index));
        ActionResult::consumed(true)
    }
    fn select(&mut self) -> ActionResult {
        let Some(selected) = self.list_state.selected() else {
            return ActionResult::not_consumed(false);
        };
        let path = &self.files[selected].path;
        let action = AsyncAction::SelectPath(path.clone(), SelectorType::PickFile);
        let _ = self.async_action_sender.as_ref().unwrap().send(action);
        self.hide();
        ActionResult::consumed(true)
    }
    fn cancel(&mut self) -> ActionResult {
        if self.list_state.selected().is_some() {
            self.list_state.select(None);
        } else {
            self.hide();
        }
        ActionResult::consumed(true)
    }
    fn map_to_list_item(selected: Option<usize>, file: &'_ FileHistory, i: usize) -> ListItem<'_> {
        let label = file.label.clone();
        let mut lines = Vec::with_capacity(3);
        let label = if selected.is_some_and(|s| s == i) {
            label.white()
        } else {
            label.dark_gray()
        };
        lines.push(label);
        if let Some(parent) = &file.parent_label {
            let label = "   ".to_string() + parent;
            lines.push(label.dark_gray().italic());
        }
        let line = Line::from(lines);
        ListItem::new(line)
    }
}

impl Component for FileHistoryComponent {
    fn register_config(&mut self, config: &Config, parent_comp: &AppComponent) {
        self.data_dir = config.config.data_dir.clone();
        self.keybinds.register_keybinds(parent_comp, config);
        let _ = parent_comp;
    }
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.effect_runner
            .register_async_action_sender(sender.clone());
        self.preview_component
            .register_async_action_sender(sender.clone());
        self.async_action_sender = Some(sender)
    }
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        let _ = key_event;
        if self.opened {
            Some(&AppComponent::FileDialog)
        } else {
            None
        }
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        if !self.showing() {
            return ActionResult::not_consumed(false);
        }
        let prev = self.preview_component.handle_action(action);
        if prev.is_consumed() {
            return prev;
        }
        match action {
            Action::Up => return self.move_up(),
            Action::Down => return self.move_down(),
            Action::Confirm => return self.select(),
            Action::Cancel => return self.cancel(),
            _ => {}
        }
        ActionResult::consumed(false)
    }
    fn handle_async_action(&mut self, action: &AsyncAction) -> ActionResult {
        if !self.showing() {
            return ActionResult::not_consumed(false);
        }
        self.preview_component.handle_async_action(action)
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.showing() {
            return;
        }
        let area = center_horizontally(area, Constraint::Percentage(60));
        let area = center_vertically(area, Constraint::Percentage(60));
        frame.render_widget(Clear, area);
        let title = Line::raw(" File History ").centered();
        let up_down_title = format!(" {} {} ", self.keybinds.up, self.keybinds.down);
        let up_down_title = Line::from(up_down_title).centered();
        let mut block = default_block().title_top(title).title_bottom(up_down_title);
        if self.list_state.selected().is_some() {
            let label = key_label_format(&self.keybinds.confirm, "Open");
            let enter_title = Line::from(label).right_aligned();
            let label = key_label_format(&self.keybinds.cancel, "Cancel");
            let cancel_title = Line::from(label).left_aligned();
            block = block.title_bottom(enter_title).title_bottom(cancel_title);
        }
        if self.files.is_empty() {
            let block_area = block.inner(area);
            let center = center(block_area);
            let text = Text::raw("No files have been opened yet...").centered();
            frame.render_widget(block, area);
            frame.render_widget(text, center);
        } else {
            let selected = self.list_state.selected();
            let mapped = self
                .files
                .iter()
                .enumerate()
                .map(|(i, file)| Self::map_to_list_item(selected, file, i));
            let list = List::new(mapped)
                .direction(ListDirection::TopToBottom)
                .highlight_symbol(HIGHLIGHT_SYMBOL)
                .highlight_spacing(HighlightSpacing::Always)
                .scroll_padding(5)
                .block(block);
            let list_area = render_preview_if_able(
                frame,
                area,
                &mut self.preview_component,
                self.list_state.selected().is_some(),
            );
            frame.render_stateful_widget(list, list_area, &mut self.list_state);
        }
        self.effect_runner.render(frame, area);
    }
}
