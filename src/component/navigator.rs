use crate::action::{Action, ActionResult, ActionSender, AsyncAction, AsyncActionSender};
use crate::component::editor::component::EditorComponent;
use crate::component::home::HomeComponent;
use crate::component::{AppComponent, Component};
use crate::config::effects::{enter_next_screen_effect, init_effect, leave_effect};
use crate::config::effects_config::EffectRunner;
use crate::config::Config;
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::layout::Rect;
use ratatui::style::{Color, Stylize};
use ratatui::widgets::Block;
use ratatui::Frame;

pub struct NavigatorComponent {
    pub current_component: AppComponent,
    pub previous_component: Option<AppComponent>,
    component: Box<dyn Component>,
    action_sender: Option<ActionSender>,
    async_action_sender: Option<AsyncActionSender>,
    effect_runner: EffectRunner,
    transitioning: Option<AppComponent>,
    config: Config,
}

impl NavigatorComponent {
    pub fn new() -> Self {
        Self::new_with_starting_component(AppComponent::default())
    }
    pub fn new_with_starting_component(app_component: AppComponent) -> Self {
        let (app_comp, comp) = Self::map_component(app_component);
        Self {
            component: comp,
            current_component: app_comp,
            previous_component: None,
            action_sender: None,
            async_action_sender: None,
            config: Config::default(),
            effect_runner: EffectRunner::default(),
            transitioning: None,
        }
    }
    pub fn navigate(&mut self, app_component: AppComponent) {
        if self.current_component != app_component {
            self.start_leave_screen_transition(app_component);
        }
    }
    pub fn return_last_component(&mut self) -> bool {
        if let Some(previous_component) = self.previous_component.take() {
            self.start_leave_screen_transition(previous_component);
            true
        } else if self.current_component != AppComponent::HomeScreen {
            self.start_leave_screen_transition(AppComponent::HomeScreen);
            true
        } else {
            false
        }
    }
    fn start_leave_screen_transition(&mut self, app_component: AppComponent) {
        self.effect_runner.add_effect(leave_effect());
        self.transitioning = Some(app_component);
    }
    fn start_enter_screen_transition(&mut self) {
        let Some(app_component) = self.transitioning.take() else {
            return;
        };
        self.effect_runner.add_effect(enter_next_screen_effect());
        self.component.exit();
        let (app_comp, comp) = Self::map_component(app_component);
        self.current_component = app_comp;
        self.component = comp;
        self.component.register_config(&self.config);
        let action_sender = self.action_sender.as_ref().unwrap();
        self.component.register_action_sender(action_sender.clone());
        let async_action_sender = self.async_action_sender.clone().unwrap();
        self.component
            .register_async_action_sender(async_action_sender);
        self.component.init();
    }
    fn map_component(app_component: AppComponent) -> (AppComponent, Box<dyn Component>) {
        match app_component {
            AppComponent::OpenedEditor(path) => {
                (AppComponent::Editor, Box::new(EditorComponent::new(path)))
            }
            AppComponent::Editor => (app_component, Box::new(EditorComponent::default())),
            _ => (AppComponent::HomeScreen, Box::new(HomeComponent::new())),
        }
    }
}

impl Component for NavigatorComponent {
    fn register_config(&mut self, config: &Config) {
        self.config = config.clone();
        self.component.register_config(config);
    }
    fn register_action_sender(&mut self, sender: ActionSender) {
        self.component.register_action_sender(sender.clone());
        self.action_sender = Some(sender);
    }
    fn register_async_action_sender(&mut self, sender: AsyncActionSender) {
        self.async_action_sender = Some(sender.clone());
        self.effect_runner.register_async_sender(sender.clone());
        self.component.register_async_action_sender(sender)
    }
    fn override_keybind_id(&self, key_event: KeyEvent) -> Option<&AppComponent> {
        self.component.override_keybind_id(key_event)
    }
    fn handle_action(&mut self, action: &Action) -> ActionResult {
        self.component.handle_action(action)
    }
    fn handle_async_action(&mut self, action: &AsyncAction) -> ActionResult {
        if let AsyncAction::Navigate(comp) = action {
            if let Some(component) = comp {
                self.navigate(component.clone())
            } else if !self.return_last_component()
                && let Some(sender) = &self.action_sender
            {
                let _ = sender.send(Action::Quit);
                return ActionResult::consumed(false);
            }
            return ActionResult::consumed(true);
        }
        self.component.handle_async_action(action)
    }
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> ActionResult {
        self.component.handle_mouse_event(mouse_event)
    }
    fn init(&mut self) {
        self.effect_runner.add_effect(init_effect());
        self.component.init();
    }
    fn exit(&mut self) {
        self.previous_component = None;
        self.component.exit();
    }
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default().bg(Color::from_u32(0x1d2021));
        frame.render_widget(block, area);
        self.component.render(frame, area);
        let just_finished = self.effect_runner.process(frame.buffer_mut(), area);
        if just_finished {
            self.start_enter_screen_transition()
        }
    }
}
