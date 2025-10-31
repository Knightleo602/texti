use crate::action::{AsyncAction, AsyncActionSender};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use std::time::Instant;
use tachyonfx::fx::{fade_from, slide_in};
use tachyonfx::{Effect, EffectManager, Interpolation, Motion};

pub fn fade_from_effect() -> Effect {
    fade_from(Color::Gray, Color::Gray, (1_000, Interpolation::SineIn))
}

pub fn slide_in_effect() -> Effect {
    let c = Color::from_u32(0x1d2021);
    let timer = (1000, Interpolation::Linear);
    slide_in(Motion::UpToDown, 10, 0, c, timer)
}

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
        self.running = true;
    }
    pub fn process(&mut self, buffer: &mut Buffer, area: Rect) {
        let elapsed = self.last_frame.elapsed();
        self.last_frame = Instant::now();
        self.effect_manager
            .process_effects(elapsed.into(), buffer, area);
        if self.running && !self.effect_manager.is_running() {
            self.running = false;
            let Some(sender) = self.async_sender.as_ref() else {
                return;
            };
            let _ = sender.send(AsyncAction::StopAnimation);
        }
    }
}
