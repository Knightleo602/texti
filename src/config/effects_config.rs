use crate::action::{AsyncAction, AsyncActionSender};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use std::time::Instant;
use tachyonfx::{Effect, EffectManager};

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
    pub fn register_async_sender(&mut self, async_sender: AsyncActionSender) {
        self.async_sender = Some(async_sender);
    }
    pub fn is_running(&self) -> bool {
        self.running
    }
    pub fn add_effect(&mut self, effect: Effect) {
        self.effect_manager.add_effect(effect);
        let _ = self
            .require_async_sender()
            .send(AsyncAction::StartAnimation);
        if !self.running {
            self.last_frame = Instant::now();
        }
        self.running = true;
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
            self.running = false;
            let Some(sender) = self.async_sender.as_ref() else {
                return true;
            };
            let _ = sender.send(AsyncAction::StopAnimation);
            true
        } else {
            false
        }
    }
    fn require_async_sender(&mut self) -> &AsyncActionSender {
        let Some(sender) = self.async_sender.as_ref() else {
            panic!("No async sender configured in effect runner");
        };
        sender
    }
}
