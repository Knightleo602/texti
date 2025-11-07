use crate::action::{
    Action, ActionResult, ActionSender, AsyncAction, AsyncActionSender, SelectorType,
};
use crate::component::component_utils::{center_horizontally, default_block, key_label_format};
use crate::component::file_selector::component::FileSelectorComponent;
use crate::component::file_selector::file_history::FileHistoryComponent;
use crate::component::{AppComponent, Component};
use crate::config::keybindings::Keybindings;
use crate::config::{get_config_file_dir, Config};
use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{HighlightSpacing, List, ListDirection, ListItem, ListState};
use ratatui::Frame;
use std::collections::HashMap;
use std::env::current_dir;
use strum::{EnumCount, EnumIter, EnumProperty, IntoEnumIterator};
use tui_big_text::{BigText, PixelSize};

#[derive(Default)]
struct HomeKeybinds {
    up: String,
    down: String,
    cancel: String,
    confirm: String,
    options_keys: HashMap<HomeOptions, String>,
}

impl HomeKeybinds {
    pub fn setup(&mut self, app_component: &AppComponent, keybinds: &Keybindings) {
        self.up = keybinds.get_key_string_or_default(Action::Up, app_component);
        self.down = keybinds.get_key_string_or_default(Action::Down, app_component);
        self.confirm = keybinds.get_key_string_or_default(Action::Confirm, app_component);
        self.cancel = keybinds.get_key_string_or_default(Action::Cancel, app_component);
        let quit = keybinds.get_key_string_or_default(Action::Quit, app_component);
        let new_file = keybinds.get_key_string_or_default(Action::NewFile, app_component);
        let open_file = keybinds.get_key_string_or_default(Action::OpenFile, app_component);
        let file_history = keybinds.get_key_string_or_default(Action::FileHistory, app_component);
        let config = keybinds.get_key_string_or_default(Action::Config, app_component);
        self.options_keys.insert(HomeOptions::NewFile, new_file);
        self.options_keys.insert(HomeOptions::OpenFile, open_file);
        self.options_keys
            .insert(HomeOptions::FileHistory, file_history);
        self.options_keys.insert(HomeOptions::Config, config);
        self.options_keys.insert(HomeOptions::Quit, quit);
    }
}

#[derive(EnumIter, EnumCount, EnumProperty, Eq, PartialEq, Clone, Copy, Hash)]
enum HomeOptions {
    #[strum(props(title = "New File"))]
    NewFile,
    #[strum(props(title = "Open File"))]
    OpenFile,
    #[strum(props(title = "File History"))]
    FileHistory,
    #[strum(props(title = "Config"))]
    Config,
    #[strum(props(title = "Quit"))]
    Quit,
}

#[derive(Default)]
pub struct HomeComponent<'a> {
    list_state: ListState,
    action_sender: Option<ActionSender>,
    async_action_sender: Option<AsyncActionSender>,
    file_selector_component: FileSelectorComponent<'a>,
    file_history_component: FileHistoryComponent,
    keybinds: HomeKeybinds,
}

impl HomeComponent<'_> {
    pub fn new() -> Self {
        HomeComponent::default()
    }
    fn navigate_new_file(&self) {
        let comp = AppComponent::Editor;
        let action = AsyncAction::Navigate(Some(comp));
        self.send_async_action(action);
    }
    fn navigate_to_config(&self) {
        let config_dir = get_config_file_dir().display().to_string();
        let comp = AppComponent::OpenedEditor(config_dir);
        self.send_async_action(AsyncAction::Navigate(Some(comp)));
    }
    fn exit_program(&self) {
        self.send_action(Action::Quit)
    }
    fn send_action(&self, action: Action) {
        let _ = self.action_sender.as_ref().unwrap().send(action);
    }
    fn send_async_action(&self, action: AsyncAction) {
        let _ = self.async_action_sender.as_ref().unwrap().send(action);
    }
    fn open_file_picker(&mut self) {
        self.file_selector_component
            .show(current_dir().unwrap_or_default(), SelectorType::PickFile)
    }
    fn open_file_history(&mut self) {
        let _ = self.file_history_component.show();
    }
}

impl Component for HomeComponent<'_> {
    fn register_config(&mut self, config: &Config, parent_comp: &AppComponent) {
        let _ = parent_comp;
        self.file_history_component
            .register_config(config, &AppComponent::HomeScreen);
        self.keybinds
            .setup(&AppComponent::HomeScreen, &config.keybindings);
    }
    fn register_action_sender(&mut self, sender: ActionSender) {
        self.file_selector_component
            .register_action_sender(sender.clone());
        self.action_sender = Some(sender);
    }
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.file_selector_component
            .register_async_action_sender(sender.clone());
        self.file_history_component
            .register_async_action_sender(sender.clone());
        self.async_action_sender = Some(sender)
    }
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        self.file_selector_component
            .override_keybind_id(key_event)
            .or_else(|| self.file_history_component.override_keybind_id(key_event))
            .or(Some(&AppComponent::HomeScreen))
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        let r = self.file_selector_component.handle_action(action);
        if r.is_consumed() {
            return r;
        }
        let r = self.file_history_component.handle_action(action);
        if r.is_consumed() {
            return r;
        }
        match action {
            Action::Up => {
                if let Some(index) = self.list_state.selected() {
                    if index > 0 {
                        self.list_state.select(Some(index - 1));
                        return ActionResult::consumed(true);
                    }
                } else {
                    self.list_state.select(Some(HomeOptions::COUNT - 1));
                    return ActionResult::consumed(true);
                }
            }
            Action::Down => {
                if let Some(index) = self.list_state.selected() {
                    if index + 1 < HomeOptions::COUNT {
                        self.list_state.select(Some(index + 1));
                        return ActionResult::consumed(true);
                    }
                } else {
                    self.list_state.select(Some(0));
                    return ActionResult::consumed(true);
                }
            }
            Action::Confirm => {
                if let Some(index) = self.list_state.selected() {
                    let option = HomeOptions::iter().get(index).unwrap();
                    match option {
                        HomeOptions::NewFile => self.navigate_new_file(),
                        HomeOptions::OpenFile => self.open_file_picker(),
                        HomeOptions::FileHistory => self.open_file_history(),
                        HomeOptions::Quit => self.exit_program(),
                        HomeOptions::Config => self.navigate_to_config(),
                    }
                    return ActionResult::consumed(true);
                }
            }
            Action::Cancel => {
                if self.list_state.selected().is_some() {
                    self.list_state.select(None);
                    return ActionResult::consumed(true);
                }
            }
            Action::NewFile => {
                self.navigate_new_file();
                return ActionResult::consumed(true);
            }
            Action::OpenFile => {
                self.open_file_picker();
                return ActionResult::consumed(true);
            }
            Action::FileHistory => {
                self.open_file_history();
                return ActionResult::consumed(true);
            }
            Action::Config => {
                self.navigate_to_config();
                return ActionResult::consumed(true);
            }
            _ => {}
        };
        Default::default()
    }
    fn handle_async_action(&mut self, action: &AsyncAction) -> ActionResult {
        let f = self.file_selector_component.handle_async_action(action);
        if f.is_consumed() {
            return f;
        }
        let r = self.file_history_component.handle_async_action(action);
        if r.is_consumed() {
            return r;
        }
        if let AsyncAction::SelectPath(path, _) = action {
            let path = path.display().to_string();
            let editor = AppComponent::OpenedEditor(path);
            let action = AsyncAction::Navigate(Some(editor));
            let _ = self.async_action_sender.as_ref().unwrap().send(action);
            return ActionResult::consumed(false);
        }
        Default::default()
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block_title = format!(" {} {} ", self.keybinds.up, self.keybinds.down);
        let block_title = Line::raw(block_title).centered();
        let mut block = default_block().title_bottom(block_title);
        if self.list_state.selected().is_some() {
            let confirm_title = key_label_format(&self.keybinds.confirm, "Select");
            let confirm_title = Line::raw(confirm_title).right_aligned();
            let cancel_title = key_label_format(&self.keybinds.cancel, "Cancel");
            let cancel_title = Line::raw(cancel_title).left_aligned();
            block = block.title_bottom(confirm_title).title_bottom(cancel_title);
        }
        let block_area = block.inner(area);
        let center_horizontal_area = center_horizontally(block_area, Constraint::Percentage(25));
        frame.render_widget(block, area);
        let [title_area, options_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length((HomeOptions::COUNT * 2) as u16),
            ])
            .flex(Flex::SpaceAround)
            .areas(center_horizontal_area);
        let title = BigText::builder()
            .centered()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new())
            .lines(vec!["Texti".into()])
            .build();
        frame.render_widget(title, title_area);
        let options_items = HomeOptions::iter().enumerate().map(|(i, v)| {
            let title = v.get_str("title").unwrap().to_string();
            let main_style = if self.list_state.selected().is_some_and(|v| v == i) {
                Style::new().white()
            } else {
                Style::new().dark_gray()
            };
            let msg = if let Some(key) = self.keybinds.options_keys.get(&v) {
                let title_span = Span::from(title).style(main_style);
                let span = Line::from(vec![(key.to_string() + "  ").dark_gray(), title_span]);
                Text::from(vec![span, Line::raw("")])
            } else {
                Text::from(format!("\n{title}\n")).style(main_style)
            };
            let msg = msg.alignment(Alignment::Center);
            ListItem::from(msg)
        });
        let list = List::new(options_items)
            .direction(ListDirection::TopToBottom)
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(list, options_area, &mut self.list_state);
        self.file_selector_component.render(frame, area);
        self.file_history_component.render(frame, area);
    }
}
