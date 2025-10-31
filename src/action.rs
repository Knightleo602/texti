use crate::component::AppComponent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use strum::Display;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub type ActionSender = UnboundedSender<Action>;
pub type ActionReceiver = UnboundedReceiver<Action>;
pub type AsyncActionSender = UnboundedSender<AsyncAction>;
pub type AsyncActionReceiver = UnboundedReceiver<AsyncAction>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaveFileResult {
    Saved(PathBuf),
    Error(String),
    MissingName,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Display)]
pub enum SelectorType {
    #[strum(to_string = " Pick Directory ")]
    PickFolder,
    #[strum(to_string = " Open File ")]
    #[default]
    PickFile,
    #[strum(to_string = " New File ")]
    NewFile,
}

impl SelectorType {
    pub fn show_files(&self) -> bool {
        self == &SelectorType::PickFile
    }
    pub fn can_pick_folder(&self) -> bool {
        self != &SelectorType::PickFile
    }
}

/// A user created action to be performed on the application
///
/// This is the action performed by the user via key events, depending on the keybind configuration
///
/// The only `Action` that is called automatically is `Action::Tick`, which is sent
/// on every tick.
///
/// Every action that is handled have an appropriate `ActionResult`, which helps
/// parent components know if the action should be consumed or passed on to other components, and
/// also to the application itself for deciding if the ui should be rerendered or not.
///
/// Generally, if the state of the component has been updated, it should rerender the terminal.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Display)]
pub enum Action {
    Character(char),
    PasteText(String),
    Save,
    Tick,
    Up,
    Down,
    Left,
    Right,
    SelectUp,
    SelectDown,
    SelectLeft,
    SelectRight,
    Confirm,
    Cancel,
    SelectAll,
    Paste,
    Cut,
    Copy,
    Insert,
    Backspace,
    Search,
    NewLine,
    Help,
    Return,
    Undo,
    Redo,
    Quit,
    Tab,
    Delete,
    OpenFile,
    Select,
    PageDown,
    PageUp,
    EndOfWord,
    StartOfWord,
}

/// Application created actions. Usually by separate tasks that have been created by `Action`s
///
/// These are used for the program to communicate with itself asynchronously, for example,
/// when finishing reading the file contents and rendering it to the screen, or after saving the
/// file contents.
///
/// This is separate from `Action` because they should not be able to be set to a specific keybind
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AsyncAction {
    LoadFileContents(String),
    SavedFile(SaveFileResult),
    /// Navigate to a component representing `AppComponent`, or return from the current one if its `None`
    Navigate(Option<AppComponent>),
    SelectPath(PathBuf, SelectorType),
    Error(String),
    StartAnimation,
    StopAnimation,
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

#[allow(dead_code)]
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
        match self {
            ActionResult::Consumed { rerender } => *rerender,
            ActionResult::NotConsumed { rerender } => *rerender,
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
