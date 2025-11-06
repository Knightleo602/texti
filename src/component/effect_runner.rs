use crate::action::{AsyncAction, AsyncActionSender};
use crate::component::Component;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Frame;
use tachyonfx::{Effect, EffectManager};
use tokio::time::Instant;

const SENDER_MISSING_ERROR_MSG: &str = "No async sender configured in effect runner";

#[derive(Debug)]
pub struct EffectRunner {
    effect_manager: EffectManager<()>,
    last_frame: Instant,
    async_sender: Option<AsyncActionSender>,
    running: bool,
}

impl Default for EffectRunner {
    fn default() -> Self {
        Self {
            effect_manager: Default::default(),
            last_frame: Instant::now(),
            async_sender: None,
            running: false,
        }
    }
}

impl EffectRunner {
    pub fn is_running(&self) -> bool {
        self.running
    }
    pub fn add_effect(&mut self, effect: Effect) {
        self.effect_manager.add_effect(effect);
        let _ = self
            .async_sender
            .as_ref()
            .expect(SENDER_MISSING_ERROR_MSG)
            .send(AsyncAction::StartAnimation);
        if !self.running {
            self.last_frame = Instant::now();
        }
        self.running = true;
    }
    pub fn cancel(&mut self) {
        self.running = false;
        let sender = self.async_sender.as_ref().expect(SENDER_MISSING_ERROR_MSG);
        let _ = sender.send(AsyncAction::StopAnimation);
    }
    /// Process the animation, if there is one running
    ///
    /// Returns true if the animation have just finished, false if the animation is still running,
    /// or there is no animation running.
    pub fn process(&mut self, buffer: &mut Buffer, area: Rect) -> bool {
        if !self.running {
            return false;
        }
        let elapsed = self.last_frame.elapsed();
        self.last_frame = Instant::now();
        self.effect_manager
            .process_effects(elapsed.into(), buffer, area);
        if !self.effect_manager.is_running() {
            self.cancel();
            true
        } else {
            false
        }
    }
}

impl Component for EffectRunner {
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.async_sender = Some(sender);
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.process(frame.buffer_mut(), area);
    }
}
