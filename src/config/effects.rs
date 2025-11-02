use ratatui::prelude::Color;
use ratatui::style::Style;
use tachyonfx::fx::{
    coalesce, coalesce_from, dissolve_to, evolve_into, fade_from, EvolveSymbolSet,
};
use tachyonfx::pattern::{RadialPattern, SweepPattern};
use tachyonfx::{Effect, Interpolation};

pub fn init_effect() -> Effect {
    let style = Style::default();
    let timer = (1000, Interpolation::ExpoInOut);
    coalesce_from(style, timer)
}

pub fn enter_next_screen_effect() -> Effect {
    let style = Style::default();
    coalesce_from(style, 300).with_pattern(SweepPattern::right_to_left(15))
}

pub fn leave_effect() -> Effect {
    dissolve_to(Style::default(), 300).with_pattern(SweepPattern::left_to_right(15))
}

pub fn fade_from_effect() -> Effect {
    fade_from(Color::Gray, Color::Gray, (1_000, Interpolation::SineIn))
}

pub fn dialog_enter(color: Color) -> Effect {
    let timer = (150, Interpolation::Linear);
    let style = Style::default().bg(color).fg(color);
    evolve_into((EvolveSymbolSet::Shaded, style), timer)
        .with_pattern(RadialPattern::center().with_transition_width(20.0))
}

pub fn show_notification_effect() -> Effect {
    coalesce(200).with_pattern(SweepPattern::up_to_down(0))
}
