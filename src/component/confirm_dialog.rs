use crate::action::{Action, ActionResult, ActionSender, AsyncActionSender};
use crate::component::component_utils::{center, default_block};
use crate::component::effect_runner::EffectRunner;
use crate::component::{AppComponent, Component};
use crate::config::effects::show_notification_effect;
use crate::config::keybindings::key_event_to_string;
use crate::config::Config;
use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::text::{Line, Text};
use ratatui::Frame;

#[derive(Default)]
pub struct ConfirmDialogComponent {
    title: String,
    message: String,
    action_on_confirm: Option<Action>,
    action_sender: Option<ActionSender>,
    effect_runner: EffectRunner,
    cancel_key: String,
    confirm_key: String,
}

impl ConfirmDialogComponent {
    pub fn show<S: ToString>(&mut self, title: S, message: S, action_on_confirm: Action) {
        self.title = title.to_string();
        self.message = message.to_string();
        self.action_on_confirm = Some(action_on_confirm);
        self.effect_runner.add_effect(show_notification_effect())
    }
    pub fn visible(&self) -> bool {
        self.action_on_confirm.is_some()
    }
}

impl Component for ConfirmDialogComponent {
    fn register_config(&mut self, config: &Config, app_component: &AppComponent) {
        let _ = app_component;
        let confirm_key = config
            .keybindings
            .get_key_event_of_action(&AppComponent::Dialog, Action::Confirm);
        self.confirm_key = confirm_key.map(key_event_to_string).unwrap_or_default();
        let cancel_key = config
            .keybindings
            .get_key_event_of_action(&AppComponent::Dialog, Action::Cancel);
        self.cancel_key = cancel_key.map(key_event_to_string).unwrap_or_default();
    }
    fn register_action_sender(&mut self, sender: ActionSender) {
        self.action_sender = Some(sender);
    }
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.effect_runner
            .register_async_action_sender(sender.clone());
    }
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        if !self.visible() {
            return None;
        };
        let _ = key_event;
        Some(&AppComponent::Dialog)
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        if !self.visible() {
            return ActionResult::not_consumed(false);
        }
        match action {
            Action::Confirm => {
                let on_confirm_action = self.action_on_confirm.take().unwrap();
                let _ = self.action_sender.as_ref().unwrap().send(on_confirm_action);
                return ActionResult::consumed(true);
            }
            Action::Cancel => {
                self.action_on_confirm = None;
                return ActionResult::consumed(true);
            }
            _ => {}
        };
        ActionResult::consumed(false)
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.visible() {
            let area = center(area);
            let enter_title = format!(" [{}] Yes ", self.confirm_key);
            let cancel_title = format!(" [{}] No ", self.cancel_key);
            let enter_title = Line::raw(&enter_title).right_aligned();
            let cancel_title = Line::raw(&cancel_title).left_aligned();
            let title = Line::raw(&self.title).centered();
            let block = default_block()
                .title_top(title)
                .title_bottom(enter_title)
                .title_bottom(cancel_title);
            let block_area = block.inner(area);
            let text_area = center(block_area);
            let text = Text::raw(&self.message).centered();
            frame.render_widget(text, text_area);
            frame.render_widget(block, area);
            self.effect_runner.process(frame.buffer_mut(), area);
        }
    }
}
