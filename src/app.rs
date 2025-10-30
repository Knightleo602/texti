use crate::action::{
    Action, ActionReceiver, ActionResult, ActionSender, AsyncAction, AsyncActionReceiver,
    AsyncActionSender,
};
use crate::component::navigator::NavigatorComponent;
use crate::component::{AppComponent, Component};
use crate::config::Config;
use crate::event::Event;
use crate::tui::Tui;
use color_eyre::Result;
use crossterm::event::{KeyEvent, MouseEvent};
use tokio::sync::mpsc;

pub struct App {
    config: Config,
    tui: Tui,
    should_quit: bool,
    action_sender: ActionSender,
    action_receiver: ActionReceiver,
    async_action_sender: AsyncActionSender,
    async_action_receiver: AsyncActionReceiver,
    component: NavigatorComponent,
    should_rerender: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let comp = NavigatorComponent::new();
        Self::create(comp)
    }

    /// Opens the file directly in the Editor component, or in the home component
    /// if `file_path` is `None`.
    pub fn new_in_editor(file_path: Option<String>) -> Result<Self> {
        let Some(file_path) = file_path else {
            return Self::new();
        };
        let editor = AppComponent::OpenedEditor(file_path);
        let comp = NavigatorComponent::new_with_starting_component(editor);
        Self::create(comp)
    }

    fn create(app_component: NavigatorComponent) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<Action>();
        let (async_action_tx, async_action_rx) = mpsc::unbounded_channel::<AsyncAction>();
        Ok(Self {
            config: Config::new()?,
            tui: Tui::new()?,
            should_quit: false,
            action_sender: action_tx,
            action_receiver: action_rx,
            async_action_receiver: async_action_rx,
            async_action_sender: async_action_tx,
            component: app_component,
            should_rerender: true,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        self.tui.enter()?;
        self.component.register_config(&self.config);
        self.component
            .register_action_sender(self.action_sender.clone());
        self.component
            .register_async_action_sender(self.async_action_sender.clone());
        self.component.init();
        self.render()?;
        loop {
            self.handle_event().await?;
            self.handle_action()?;
            self.handle_async_action()?;
            if self.should_quit {
                break;
            }
        }
        self.component.exit();
        self.tui.exit()?;
        Ok(())
    }

    async fn handle_event(&mut self) -> Result<()> {
        let Some(event) = self.tui.event_receiver.recv().await else {
            return Ok(());
        };
        match event {
            Event::Quit => self.should_quit = true,
            Event::Tick => self.action_sender.send(Action::Tick)?,
            Event::Render => self.render()?,
            Event::Resize(_, _) => self.should_rerender = true,
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event)?,
            Event::Key(event) => self.handle_key_event(event)?,
            Event::Paste(text) => self.action_sender.send(Action::PasteText(text))?,
            Event::Error(msg) => self.async_action_sender.send(AsyncAction::Error(msg))?,
            _ => {}
        };
        Ok(())
    }

    fn handle_action(&mut self) -> Result<()> {
        while let Ok(action) = self.action_receiver.try_recv() {
            let res = match action {
                Action::Quit => {
                    self.should_quit = true;
                    return Ok(());
                }
                _ => self.component.handle_action(action),
            };
            self.flag_for_rerender_if_asked(res);
        }
        Ok(())
    }
    fn handle_async_action(&mut self) -> Result<()> {
        while let Ok(action) = self.async_action_receiver.try_recv() {
            let res = self.component.handle_async_action(action);
            self.flag_for_rerender_if_asked(res)
        }
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let comp = self
            .component
            .override_keybind_id(key_event)
            .unwrap_or(&self.component.current_component);
        let action = if let Some(action) = self.config.keybindings.get_action(comp, key_event) {
            action
        } else if let Some(char) = key_event.code.as_char() {
            Action::Character(char)
        } else {
            return Ok(());
        };
        self.action_sender.send(action)?;
        Ok(())
    }
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> Result<()> {
        let result = self.component.handle_mouse_event(mouse_event);
        self.flag_for_rerender_if_asked(result);
        Ok(())
    }
    fn flag_for_rerender_if_asked(&mut self, action_result: ActionResult) {
        if !self.should_rerender && action_result.should_rerender() {
            self.should_rerender = true;
        };
    }
    fn render(&mut self) -> Result<()> {
        if self.should_rerender {
            self.tui
                .terminal
                .draw(|frame| self.component.render(frame, frame.area()))?;
            self.should_rerender = false;
        }
        Ok(())
    }
}
