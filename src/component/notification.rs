use crate::action::{Action, ActionResult};
use crate::component::component_utils::{center_horizontally, default_block};
use crate::component::{Component, TickCount};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Color, Text};
use ratatui::widgets::{Clear, Paragraph};
use ratatui::Frame;

#[derive(Default, Debug)]
pub struct Notification {
    title: TickCount<String>,
    error: bool,
}

#[derive(Debug, Default)]
pub struct NotificationComponent {
    notification: Option<Notification>,
}

impl NotificationComponent {
    pub fn new(text: String, error: bool) -> Self {
        let notification = Notification {
            title: TickCount::new(text),
            error,
        };
        Self {
            notification: Some(notification),
        }
    }
    pub fn new_with_count(text: String, error: bool, count: usize) -> Self {
        let notification = Notification {
            title: TickCount { value: text, count },
            error,
        };
        Self {
            notification: Some(notification),
        }
    }
    pub fn notify_text<T: ToString>(&mut self, text: T) {
        let notification = Notification {
            title: TickCount::new(text.to_string()),
            error: false,
        };
        self.notification = Some(notification);
    }
    pub fn notify_error<T: ToString>(&mut self, text: T) {
        let notification = Notification {
            title: TickCount::new(text.to_string()),
            error: true,
        };
        self.notification = Some(notification);
    }
    pub fn notify(&mut self, notification: Notification) {
        self.notification = Some(notification);
    }
    pub fn handle_tick_action(&mut self) -> ActionResult {
        if let Some(count) = &mut self.notification {
            if count.title.countdown() {
                self.notification = None;
            }
            ActionResult::consumed(true)
        } else {
            Default::default()
        }
    }
    pub fn handle_action_ref(&mut self, action: &Action) -> ActionResult {
        match action {
            Action::Tick => self.handle_tick_action(),
            Action::Cancel => {
                if self.notification.take().is_some() {
                    ActionResult::consumed(true)
                } else {
                    Default::default()
                }
            }
            _ => Default::default(),
        }
    }
}

impl Component for NotificationComponent {
    fn handle_action(&mut self, action: Action) -> ActionResult {
        self.handle_action_ref(&action)
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(counter) = &self.notification {
            let string_len = (counter.title.value.len() + 4) as u16;
            let pop_up_area = center_horizontally(area, Constraint::Length(string_len));
            let [pop_up_area] = Layout::vertical([Constraint::Length(3)])
                .flex(Flex::End)
                .vertical_margin(1)
                .areas(pop_up_area);
            frame.render_widget(Clear, pop_up_area);
            let text = Text::raw(&counter.title.value);
            let mut paragraph = Paragraph::new(text).centered();
            let block = default_block();
            if counter.error {
                let block = block.border_style(Color::Red);
                paragraph = paragraph.style(Color::Red).block(block);
            } else {
                paragraph = paragraph.block(block);
            };
            frame.render_widget(paragraph, pop_up_area);
        }
    }
}
