use crate::component::AppComponent;
use serde::Deserialize;
use std::path::PathBuf;
use strum::{Display, EnumDiscriminants};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type ActionSender = UnboundedSender<Action>;
pub type ActionReceiver = UnboundedReceiver<Action>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub enum SaveFileResult {
    Saved(PathBuf),
    Error(String),
    MissingName,
}

/// An Action to be performed on the application
///
/// This can be action given by the user (via key events) or from the
/// program itself (when communicating asynchronously for example).
///
/// The only `Action` that is called automatically is `Action::Tick`, which is sent
/// on every tick.
///
/// Every action that is handled should hav an appropriate `ActionResult`, which helps
/// parent components know if the action should be consumed or passed on to other components, and
/// also to the application itself for deciding if the ui should be rerendered or not.
///
/// Generally, if the state of the component has been updated, it should rerender the terminal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, EnumDiscriminants, Display)]
pub enum Action {
    LoadFileContents(String),
    PasteEvent(String),
    Save,
    SavedFile(SaveFileResult),
    /// Navigate to a component representing `AppComponent`, or return if its `None`
    Navigate(Option<AppComponent>),
    Error(String),
    Tick,
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    SelectAll,
    Paste,
    Cut,
    Copy,
    Insert,
    Help,
    Return,
    Undo,
    Redo,
    Quit,
}

impl Action {
    pub fn is_directional_action(&self) -> bool {
        matches!(
            self,
            Action::Up | Action::Down | Action::Left | Action::Right
        )
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ActionResult {
    Consumed { rerender: bool },
    NotConsumed { rerender: bool },
}

impl ActionResult {
    pub fn is_consumed(&self) -> bool {
        matches!(self, Self::Consumed { .. })
    }
    pub fn is_consumed_and_rerender(&self) -> bool {
        matches!(self, Self::Consumed { rerender: true })
    }
    pub fn is_not_consumed(&self) -> bool {
        matches!(self, Self::NotConsumed { .. })
    }
    pub fn should_rerender(&self) -> bool {
        match *self {
            ActionResult::Consumed { rerender } => rerender,
            ActionResult::NotConsumed { rerender } => rerender,
        }
    }
    #[inline]
    pub fn consumed(rerender: bool) -> Self {
        ActionResult::Consumed { rerender }
    }
    #[inline]
    pub fn not_consumed(rerender: bool) -> Self {
        ActionResult::NotConsumed { rerender }
    }
}

impl Default for ActionResult {
    fn default() -> Self {
        ActionResult::not_consumed(false)
    }
}
