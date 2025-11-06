use crate::action::{Action, ActionResult, AsyncActionSender};
use crate::component::component_utils::default_block;
use crate::component::effect_runner::EffectRunner;
use crate::component::{AppComponent, Component};
use crate::config::effects::floating_component_bottom_right_enter;
use crate::config::keybindings::key_event_to_string;
use crate::config::Config;
use color_eyre::eyre::{OptionExt, Result};
use crossterm::event::KeyEvent;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::cmp::Ordering;
use std::fmt::Display;

#[derive(PartialEq, Clone, Debug)]
pub struct KeyBind {
    pub key: String,
    pub label: String,
    pub action: Action,
}

impl From<(&KeyEvent, &Action)> for KeyBind {
    fn from(value: (&KeyEvent, &Action)) -> KeyBind {
        let (key, action) = value;
        let string_key = key_event_to_string(key);
        let action_string = action.to_string();
        Self {
            key: string_key,
            label: action_string,
            action: action.clone(),
        }
    }
}

impl<'a> From<&KeyBind> for Line<'a> {
    fn from(value: &KeyBind) -> Self {
        Line::from(vec![
            Span::raw("["),
            Span::from(value.key.to_string()).gray(),
            Span::raw("] "),
            Span::from(value.label.to_string()).white(),
        ])
    }
}

impl Display for KeyBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " [{}] {} ", self.key, self.label)
    }
}

/// A dialog component for showing all possible keybinds in a [`AppComponent`].
///
/// Pass the entire layout area when rendering this component, it positions itself always
/// at the bottom right of the screen.
///
/// You must register the config and the async action sender to this component, in order to
/// obtain the parent`s keybinds and run the transition effect when showing.
///
/// This component is also responsible for handling actions while visible, so you should also
/// pass the action to it as well.
///
/// You can optionally add an [`ActionSender`] to transform this component into a sort of
/// command pallet
#[derive(Debug)]
pub struct HelpComponent {
    pub title: String,
    help_key: String,
    keybinds: Vec<KeyBind>,
    width: u16,
    visible: bool,
    effect_runner: EffectRunner,
    scroll_offset: u16,
    max_offset: u16,
}

impl Default for HelpComponent {
    fn default() -> Self {
        Self {
            title: String::from(" Help "),
            keybinds: vec![],
            visible: false,
            width: 0,
            help_key: String::new(),
            effect_runner: EffectRunner::default(),
            scroll_offset: 0,
            max_offset: 0,
        }
    }
}

#[allow(dead_code)]
impl HelpComponent {
    pub fn new<S: AsRef<str>>(title: S, keybinds: Vec<KeyBind>) -> Self {
        let title = title.as_ref();
        let title = if title.is_empty() {
            String::from(" Help ")
        } else {
            format!(" Help - {} ", title)
        };
        let mut n = Self {
            title,
            keybinds: vec![],
            visible: false,
            width: 0,
            help_key: String::new(),
            effect_runner: EffectRunner::default(),
            scroll_offset: 0,
            max_offset: 0,
        };
        n.register_keybinds(keybinds);
        n
    }
    pub fn from_component<S: AsRef<str>>(
        title: S,
        app_component: AppComponent,
        config: &Config,
    ) -> Option<Self> {
        let keybinds = config.keybindings.get_all_keybinds(&app_component)?;
        let c = keybinds.map(KeyBind::from).collect();
        Some(Self::new(title, c))
    }
    pub fn register_from_app_component(
        &mut self,
        app_component: &AppComponent,
        config: &Config,
    ) -> Result<()> {
        let keybinds = config
            .keybindings
            .get_all_keybinds(app_component)
            .ok_or_eyre("App component has not keybinds set")?;
        let c = keybinds.map(KeyBind::from).collect();
        self.register_keybinds(c);
        Ok(())
    }
    pub fn help_key(&self) -> &str {
        &self.help_key
    }
    fn register_keybinds(&mut self, keybinds: Vec<KeyBind>) {
        self.keybinds = keybinds;
        let max = self.keybinds.iter().max_by(move |x, x1| {
            let len1 = x.to_string().len();
            let len2 = x1.to_string().len();
            if len1 == len2 {
                Ordering::Equal
            } else if len1 > len2 {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
        self.width = max.map(move |t| t.to_string().len()).unwrap_or_default() as u16 + 9;
    }
    fn block<'a>(&self) -> Block<'a> {
        let line = Line::from(self.title.to_string()).left_aligned();
        default_block().title_top(line)
    }
    pub fn toggle_visible(&mut self) -> ActionResult {
        self.visible = !self.visible;
        if self.visible {
            self.effect_runner
                .add_effect(floating_component_bottom_right_enter())
        }
        ActionResult::consumed(true)
    }
    fn scroll_up(&mut self) -> ActionResult {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            return ActionResult::consumed(true);
        }
        ActionResult::default()
    }
    fn scroll_down(&mut self) -> ActionResult {
        if self.scroll_offset < self.max_offset {
            self.scroll_offset += 1;
            return ActionResult::consumed(true);
        }
        ActionResult::default()
    }
}

impl Component for HelpComponent {
    /// Registers and obtains all the keybinds available in `parent_comp`.
    ///
    /// You can either pass down the app component or provide another one for different keybinds.
    ///
    /// This component also obtains the keybind for opening itself, you can get it
    /// by calling `help_key()`.
    fn register_config(&mut self, config: &Config, parent_comp: &AppComponent) {
        let _ = parent_comp;
        let help_key = config
            .keybindings
            .get_key_event_of_action(parent_comp, Action::ToggleHelp)
            .map(key_event_to_string);
        self.help_key = help_key.unwrap_or_default();
        let _ = self.register_from_app_component(parent_comp, config);
    }
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.effect_runner.register_async_action_sender(sender);
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        if !self.visible {
            if &Action::ToggleHelp == action {
                return self.toggle_visible();
            }
            return ActionResult::not_consumed(false);
        }
        match action {
            Action::Up => return self.scroll_up(),
            Action::Down => return self.scroll_down(),
            Action::ToggleHelp => return self.toggle_visible(),
            _ => {}
        }
        ActionResult::default()
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }
        let [area] = Layout::horizontal([Constraint::Length(self.width)])
            .flex(Flex::End)
            .horizontal_margin(1)
            .areas(area);
        let [area] = Layout::vertical([Constraint::Percentage(65)])
            .flex(Flex::End)
            .areas(area);
        let mut block = self.block();
        let lines = self.keybinds.iter().map(Line::from).collect::<Vec<_>>();
        let mut paragraph = Paragraph::new(lines).scroll((self.scroll_offset, 0));
        if self.scroll_offset > 0 {
            let arrow_up = Line::raw("  ").centered();
            block = block.title_top(arrow_up);
        }
        self.max_offset = (self.keybinds.len() as u16).saturating_sub(area.height + 2);
        if self.scroll_offset < self.max_offset {
            let arrow_down = Line::raw("  ").centered();
            block = block.title_bottom(arrow_down);
        }
        paragraph = paragraph.block(block);
        frame.render_widget(paragraph, area);
        self.effect_runner.process(frame.buffer_mut(), area);
    }
}
