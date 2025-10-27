use crate::event::Event;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use crossterm::cursor;
use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event as CrosstermEvent, EventStream,
    KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

type EventReceiver = UnboundedReceiver<Event>;
type EventSender = UnboundedSender<Event>;

const TICK_DURATION: Duration = Duration::from_millis(1000);
const FRAME_DURATION: Duration = Duration::from_millis(33);

pub struct Tui {
    pub terminal: Terminal<Backend<Stdout>>,
    pub event_loop_task: JoinHandle<()>,
    pub cancellation_token: CancellationToken,
    pub event_receiver: EventReceiver,
    pub event_sender: EventSender,
}

impl Tui {
    pub fn new() -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            terminal: Terminal::new(Backend::new(stdout()))?,
            event_loop_task: tokio::spawn(async {}),
            cancellation_token: CancellationToken::new(),
            event_sender: event_tx,
            event_receiver: event_rx,
        })
    }

    pub fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnableBracketedPaste)?;
        crossterm::execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
        crossterm::execute!(
            stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
        self.start_receiving_events();
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop_receiving_events()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.terminal.flush()?;
            crossterm::terminal::disable_raw_mode()?;
        }
        crossterm::execute!(stdout(), DisableBracketedPaste)?;
        crossterm::execute!(stdout(), LeaveAlternateScreen, cursor::Show)?;
        crossterm::execute!(stdout(), PopKeyboardEnhancementFlags)?;
        Ok(())
    }

    fn start_receiving_events(&mut self) {
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let event_sender = self.event_sender.clone();
        let cancellation_token = self.cancellation_token.clone();
        self.event_loop_task = tokio::spawn(async move {
            Self::event_loop(event_sender, cancellation_token).await;
        });
    }

    fn stop_receiving_events(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.event_loop_task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.event_loop_task.abort();
            }
            if counter > 100 {
                return Err(eyre!(
                    "Failed to abort task in 100 milliseconds for unknown reason"
                ));
            }
        }
        Ok(())
    }

    fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    async fn event_loop(event_tx: UnboundedSender<Event>, cancellation_token: CancellationToken) {
        let mut event_stream = EventStream::new();
        let mut tick_interval = interval(TICK_DURATION);
        let mut render_interval = interval(FRAME_DURATION);
        // if this fails, then it's likely a bug in the calling code
        event_tx
            .send(Event::Init)
            .expect("failed to send init event");
        loop {
            let event = tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                _ = tick_interval.tick() => Event::Tick,
                _ = render_interval.tick() => Event::Render,
                crossterm_event = event_stream.next().fuse() => match crossterm_event {
                    Some(Ok(event)) => match event {
                        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Event::Key(key),
                        CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
                        CrosstermEvent::Resize(x, y) => Event::Resize(x, y),
                        CrosstermEvent::Paste(s) => Event::Paste(s),
                        _ => continue, // ignore other events
                    }
                    Some(Err(e)) => Event::Error(e.to_string()),
                    None => break, // the event stream has stopped and will not produce any more events
                },
            };
            if event_tx.send(event).is_err() {
                // the receiver has been dropped, so there's no point in continuing the loop
                break;
            }
        }
        cancellation_token.cancel();
    }
}

impl Deref for Tui {
    type Target = Terminal<Backend<Stdout>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
