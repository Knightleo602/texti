mod component_utils;
mod confirm_dialog;
mod editor;
mod effect_runner;
mod file_selector;
mod help;
mod home;
pub(crate) mod navigator;
mod notification;
mod preview_component;

use crate::action::{Action, ActionResult, ActionSender, AsyncAction, AsyncActionSender};
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
    OpenedEditor(String),
    FileDialog,
    Editor,
    Dialog,
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

#[allow(dead_code)]
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
    fn register_config(&mut self, config: &Config, parent_comp: &AppComponent) {
        let _ = parent_comp;
        let _ = config;
    }
    /// Register an `ActionSender` to the component, to be received by currently visible
    /// components and updated.
    fn register_action_sender(&mut self, sender: ActionSender) {
        let _ = sender;
    }
    /// Register `AsyncActionSender` to the component, to send asynchronous task results.
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        let _ = sender;
    }
    /// Forces the key event handler to get a key bind from another component
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        let _ = key_event;
        None
    }
    /// Handles the action and update this state
    ///
    /// Return `ActionResult::Consumed` to stop passing the action to other components,
    /// or `ActionResult::NotConsumed` to continue.
    ///
    /// The terminal will rerender itself if `ActionResult::should_rerender` returns true,
    /// so it should be true only if the component state has changed.
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        let _ = action;
        ActionResult::default()
    }
    /// Handles the async action created by the component itself, or it parents.
    ///
    /// This has the same behavior as `handle_action`
    fn handle_async_action(&mut self, action: &AsyncAction) -> ActionResult {
        let _ = action;
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
