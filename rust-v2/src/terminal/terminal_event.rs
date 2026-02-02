use crossterm::event::{Event, KeyEvent};

#[derive(Debug, Clone, Copy)]
pub enum TerminalEvent {
    Key(KeyEvent),
    Resize { width: u16, height: u16 },
}

impl TryFrom<Event> for TerminalEvent {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event {
            Event::Key(key) => Ok(TerminalEvent::Key(key)),
            Event::Resize(width, height) => Ok(TerminalEvent::Resize { width, height }),
            _ => Err(()),
        }
    }
}
