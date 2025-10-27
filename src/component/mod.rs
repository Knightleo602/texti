mod component_utils;
mod editor;
mod help;
mod home;
pub(crate) mod navigator;
mod notification;

use crate::action::{Action, ActionResult, ActionSender};
use crate::config::Config;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::Frame;
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

const TICKS_UNTIL_REMOVE_POPUP: usize = 2;

#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Hash, Serialize)]
pub enum AppComponent {
    #[default]
    HomeScreen,
    FileTree,
    OpenedEditor(String),
    Editor,
    Dialogs,
}

#[derive(Debug)]
pub(super) struct TickCount<T> {
    pub value: T,
    pub count: usize,
}

impl<T> Default for TickCount<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> TickCount<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            count: TICKS_UNTIL_REMOVE_POPUP,
        }
    }

    pub fn new_with_ticks(value: T, ticks: usize) -> Self {
        Self {
            value,
            count: ticks,
        }
    }

    pub fn countdown(&mut self) -> bool {
        if self.count == 0 {
            true
        } else {
            self.count -= 1;
            false
        }
    }
}

impl<T> Deref for TickCount<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for TickCount<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

pub trait Component {
    /// Gets a reference current `Config` data
    /// This config may clone what is needed from it and keep it in its state.
    fn register_config(&mut self, config: &Config) {
        let _ = config;
    }
    /// Register and `ActionSender` to the component, to be received by currently visible
    /// components and updated.
    fn set_action_sender(&mut self, sender: ActionSender) {
        let _ = sender;
    }
    /// Handles the action and update this state
    ///
    /// Return `ActionResult::Consumed` to stop passing the action to other components,
    /// or `ActionResult::NotConsumed` to continue.
    ///
    /// The terminal will automatically rerender itself if `ActionResult::Consumed` is returned
    /// or `ActionResult::NotConsumed.rerender` is true.
    fn handle_action(&mut self, action: Action) -> ActionResult {
        let _ = action;
        ActionResult::default()
    }
    /// Handles `KeyEvent` before they become an `Action` and sent to `handle_action`.
    /// If this returns `ActionResult::Consumed`, `handle_action` is not called.
    fn handle_key_event(&mut self, key_event: KeyEvent) -> ActionResult {
        let _ = key_event;
        ActionResult::default()
    }
    /// Handles the mouse action and update this state
    ///
    /// Return `ActionResult::Consumed` to stop passing the mouse event to other components,
    /// or `ActionResult::NotConsumed` to continue.
    ///
    /// The terminal will automatically rerender itself if `ActionResult::Consumed` is returned
    /// or `ActionResult::NotConsumed.rerender` is true.
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> ActionResult {
        let _ = mouse_event;
        ActionResult::default()
    }
    /// Initialize the component
    /// This is called when first appearing, and after setting the component configuration handler
    /// and action sender.
    fn init(&mut self) {}
    /// Called when the component is exiting the screen. Equivalent to `drop()`
    fn exit(&mut self) {}
    /// Render the component content
    fn render(&mut self, frame: &mut Frame, area: Rect);
}
