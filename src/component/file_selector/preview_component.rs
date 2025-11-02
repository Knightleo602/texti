use crate::action::{Action, ActionResult, AsyncAction, AsyncActionSender};
use crate::component::component_utils::default_block;
use crate::component::Component;
use crate::util::read_dir_limited;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::path::PathBuf;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub(super) struct PreviewComponent {
    path_buf: PathBuf,
    contents: Option<String>,
    task: JoinHandle<()>,
    async_action_sender: Option<AsyncActionSender>,
    lines: usize,
    visible: bool,
}

impl Default for PreviewComponent {
    fn default() -> Self {
        Self {
            path_buf: Default::default(),
            contents: None,
            task: tokio::spawn(async {}),
            async_action_sender: None,
            lines: 0,
            visible: true,
        }
    }
}

impl PreviewComponent {
    pub fn change_dir(&mut self, dir: Option<PathBuf>) {
        if let Some(dir) = dir {
            self.path_buf = dir;
        }
        self.contents = None;
        self.task.abort();
    }
    fn read_lines(&mut self) {
        if self.contents.is_some() {
            return;
        }
        self.task.abort();
        let action_sender = self.async_action_sender.clone().unwrap();
        let path = self.path_buf.clone();
        let lines = self.lines;
        self.task = tokio::spawn(async move {
            match read_dir_limited(&path, lines).await {
                Ok(content) => {
                    let action = AsyncAction::PreviewContents(Some(content));
                    let _ = action_sender.send(action);
                }
                Err(err) => {
                    let action = AsyncAction::Error(err.to_string());
                    let _ = action_sender.send(action);
                }
            }
        });
    }

    pub fn visible(&self) -> bool {
        self.visible && self.path_buf.is_file()
    }

    fn reload(&mut self) {
        self.contents = None;
        self.read_lines();
    }
}

impl Component for PreviewComponent {
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.async_action_sender = Some(sender)
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        match action {
            Action::TogglePreview => {
                self.visible = !self.visible;
                return ActionResult::consumed(true);
            }
            Action::ReloadPreview => {
                self.reload();
                return ActionResult::consumed(true);
            }
            Action::Resize(_, _) => {
                self.reload();
                return ActionResult::not_consumed(true);
            }
            _ => {}
        }
        Default::default()
    }

    fn handle_async_action(&mut self, action: &AsyncAction) -> ActionResult {
        if let AsyncAction::PreviewContents(contents) = action {
            self.contents = contents.clone();
            return ActionResult::consumed(true);
        }
        Default::default()
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let title = Line::raw(" Preview ").left_aligned();
        let block = default_block().title_top(title);
        let text = self.contents.clone().unwrap_or_default();
        let paragraph = Paragraph::new(text).block(block).gray();
        frame.render_widget(paragraph, area);
        self.lines = area.height as usize - 2;
        self.read_lines();
    }
}

impl Drop for PreviewComponent {
    fn drop(&mut self) {
        self.task.abort();
    }
}
