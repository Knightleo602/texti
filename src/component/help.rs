use crate::action::Action;
use crate::component::component_utils::default_block;
use crate::component::{AppComponent, Component};
use crate::config::keybindings::key_event_to_string;
use crate::config::Config;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Text, ToSpan};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;
use std::fmt::Display;

const KEYBINDS_SEPARATOR: &str = " | ";
pub const KEYBINDS_HELP_TITLE: &str = "Keybinds";

#[derive(PartialEq, Clone, Debug)]
pub struct KeyBind {
    key: String,
    label: String,
}

impl From<(&KeyEvent, &Action)> for KeyBind {
    fn from(value: (&KeyEvent, &Action)) -> KeyBind {
        let (key, action) = value;
        let string_key = key_event_to_string(key);
        let action_string = action.to_string();
        Self {
            key: string_key,
            label: action_string,
        }
    }
}

impl<'a> From<KeyBind> for Text<'a> {
    fn from(value: KeyBind) -> Self {
        value.to_string().into()
    }
}

impl Display for KeyBind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " [{}]{} ", self.key, self.label)
    }
}

#[derive(Default, Clone, Debug)]
pub struct HelpComponent {
    title: String,
    keybinds: Vec<KeyBind>,
}

impl HelpComponent {
    pub fn new<S: AsRef<str>>(title: S, keybinds: Vec<KeyBind>) -> Self {
        let title = title.as_ref();
        let title = if title.is_empty() {
            String::from(" Help ")
        } else {
            format!(" Help - {} ", title)
        };
        Self { title, keybinds }
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
    fn block<'a>(&self) -> Block<'a> {
        let line = Line::from(self.title.to_string()).centered();
        default_block().title_top(line)
    }
}

impl Component for HelpComponent {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = self.block();
        let mut line = Line::default();
        let len = self.keybinds.len();
        for (i, keybind) in self.keybinds.iter().enumerate() {
            let key_text = format!("[{}] ", keybind.key).dark_gray();
            line += key_text;
            line += keybind.label.to_span().white();
            if i < len - 1 {
                line += KEYBINDS_SEPARATOR.into()
            }
        }
        let paragraph = Paragraph::new(line).block(block);
        frame.render_widget(paragraph, area);
    }
}
