use crate::action::{
    Action, ActionResult, ActionSender, AsyncAction, AsyncActionSender, SelectorType,
};
use crate::component::component_utils::{center_horizontally, default_block};
use crate::component::file_selector::component::FileSelectorComponent;
use crate::component::{AppComponent, Component};
use crate::config::keybindings::key_event_to_string;
use crate::config::{get_config_file_dir, Config};
use crossterm::event::KeyEvent;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{HighlightSpacing, List, ListDirection, ListItem, ListState};
use ratatui::Frame;
use std::env::current_dir;
use strum::{EnumCount, EnumIter, EnumProperty, IntoEnumIterator};
use tui_big_text::{BigText, PixelSize};

#[derive(EnumIter, EnumCount, EnumProperty, Eq, PartialEq, Clone, Copy)]
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
    up_key: String,
    down_key: String,
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
}

impl Component for HomeComponent<'_> {
    fn register_config(&mut self, config: &Config, parent_comp: &AppComponent) {
        let _ = parent_comp;
        let up_key = config
            .keybindings
            .get_key_event_of_action(&AppComponent::HomeScreen, Action::Up)
            .map(key_event_to_string)
            .unwrap_or_default();
        let down_key = config
            .keybindings
            .get_key_event_of_action(&AppComponent::HomeScreen, Action::Down)
            .map(key_event_to_string)
            .unwrap_or_default();
        self.up_key = up_key;
        self.down_key = down_key;
    }
    fn register_action_sender(&mut self, sender: ActionSender) {
        self.file_selector_component
            .register_action_sender(sender.clone());
        self.action_sender = Some(sender);
    }
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.file_selector_component
            .register_async_action_sender(sender.clone());
        self.async_action_sender = Some(sender)
    }
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        let o = self.file_selector_component.override_keybind_id(key_event);
        if o.is_some() {
            o
        } else {
            Some(&AppComponent::HomeScreen)
        }
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        let r = self.file_selector_component.handle_action(action);
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
                        HomeOptions::FileHistory => return ActionResult::consumed(false),
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
            _ => {}
        };
        Default::default()
    }
    fn handle_async_action(&mut self, action: &AsyncAction) -> ActionResult {
        let f = self.file_selector_component.handle_async_action(action);
        if f.is_consumed() {
            return f;
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
        let block_title = format!(" {} {} select ", self.up_key, self.down_key);
        let block_title = Line::raw(block_title).centered();
        let block = default_block().title_bottom(block_title);
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
        let options_items = HomeOptions::iter().map(move |v| {
            let msg = v.get_str("title").unwrap().to_string();
            let msg = format!("\n{}\n", msg);
            let line = Text::from(msg).alignment(Alignment::Center).dark_gray();
            ListItem::new(line)
        });
        let list = List::new(options_items)
            .direction(ListDirection::TopToBottom)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(Style::new().white());
        frame.render_stateful_widget(list, options_area, &mut self.list_state);
        self.file_selector_component.render(frame, area);
    }
}
