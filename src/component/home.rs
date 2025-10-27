use crate::action::{Action, ActionResult, ActionSender};
use crate::component::component_utils::{center_horizontally, default_block};
use crate::component::{AppComponent, Component};
use crate::config::get_config_file_dir;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Text;
use ratatui::Frame;
use strum::{EnumCount, EnumIter, EnumProperty, IntoEnumIterator};
use tui_big_text::{BigText, PixelSize};

#[derive(EnumIter, EnumCount, EnumProperty, Eq, PartialEq, Clone, Copy)]
enum HomeOptions {
    #[strum(props(title = "New File"))]
    NewFile,
    #[strum(props(title = "Open File"))]
    OpenFile,
    #[strum(props(title = "File History"))]
    FileHistory,
    #[strum(props(title = "Config"))]
    Config,
    #[strum(props(title = "Quit"))]
    Quit,
}

#[derive(Default)]
pub struct HomeComponent {
    selected_option: Option<usize>,
    action_sender: Option<ActionSender>,
}

impl HomeComponent {
    pub fn new() -> Self {
        HomeComponent {
            ..Default::default()
        }
    }

    fn navigate_new_file(&self) {
        let comp = AppComponent::Editor;
        let action = Action::Navigate(Some(comp));
        self.send_action(action);
    }

    fn navigate_to_config(&self) {
        let config_dir = get_config_file_dir().display().to_string();
        let comp = AppComponent::OpenedEditor(config_dir);
        self.send_action(Action::Navigate(Some(comp)));
    }

    fn exit_program(&self) {
        self.send_action(Action::Quit)
    }

    fn send_action(&self, action: Action) {
        let _ = self.action_sender.as_ref().unwrap().send(action);
    }
}

impl Component for HomeComponent {
    fn set_action_sender(&mut self, sender: ActionSender) {
        self.action_sender = Some(sender);
    }
    fn handle_action(&mut self, action: Action) -> ActionResult {
        match action {
            Action::Up => {
                if let Some(index) = self.selected_option {
                    if index > 0 {
                        self.selected_option = Some(index - 1);
                        return ActionResult::consumed(true);
                    }
                } else {
                    self.selected_option = Some(HomeOptions::COUNT - 1);
                    return ActionResult::consumed(true);
                }
            }
            Action::Down => {
                if let Some(index) = self.selected_option {
                    if index + 1 < HomeOptions::COUNT {
                        self.selected_option = Some(index + 1);
                        return ActionResult::consumed(true);
                    }
                } else {
                    self.selected_option = Some(0);
                    return ActionResult::consumed(true);
                }
            }
            Action::Confirm => {
                if let Some(index) = self.selected_option {
                    let option = HomeOptions::iter().get(index).unwrap();
                    match option {
                        HomeOptions::NewFile => self.navigate_new_file(),
                        HomeOptions::OpenFile => {}
                        HomeOptions::FileHistory => {}
                        HomeOptions::Quit => self.exit_program(),
                        HomeOptions::Config => self.navigate_to_config(),
                    }
                    return ActionResult::consumed(true);
                }
            }
            Action::Cancel => {
                let taken = self.selected_option.take().is_some();
                return ActionResult::consumed(taken);
            }
            _ => {}
        };
        Default::default()
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = default_block();
        let block_area = block.inner(area);
        let center_horizontal_area = center_horizontally(block_area, Constraint::Percentage(25));
        frame.render_widget(block, area);
        let [title_area, options_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length((HomeOptions::COUNT * 2) as u16),
            ])
            .flex(Flex::SpaceAround)
            .areas(center_horizontal_area);
        let title = BigText::builder()
            .centered()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new())
            .lines(vec!["Texti".into()])
            .build();
        frame.render_widget(title, title_area);
        let options_constraints = [Constraint::Length(2); HomeOptions::COUNT];
        let options: [Rect; HomeOptions::COUNT] =
            Layout::vertical(options_constraints).areas(options_area);
        for (i, option) in HomeOptions::iter().enumerate() {
            let msg = option.get_str("title").unwrap();
            let mut text = Text::raw(msg).centered();
            let selected = self.selected_option.is_some_and(|o| o == i);
            text = if selected { text.white() } else { text.gray() };
            frame.render_widget(text, options[i]);
        }
    }
}
