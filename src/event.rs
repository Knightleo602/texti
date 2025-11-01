use crossterm::event::{KeyEvent, MouseEvent};

pub enum Event {
    Init,
    Tick,
    Render,
    Mouse(MouseEvent),
    Key(KeyEvent),
    Paste(String),
    Error(String),
    Resize(u16, u16),
}
