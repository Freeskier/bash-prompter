use crossterm::event::{KeyCode, KeyModifiers};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    /// Naciśnięcie klawisza (surowy event)
    KeyPressed {
        code: KeyCode,
        modifiers: KeyModifiers,
    },

    /// Akcja wysokopoziomowa (zmapowana z klawisza)
    Action(Action),

    /// Zmiana rozmiaru terminala
    Resize { width: u16, height: u16 },

    /// Tick (dla animacji/timerów)
    Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Exit,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Submit,
    Cancel,
}

/// Typ eventu do filtrowania subskrypcji
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    KeyPressed,
    Action,
    Resize,
    Tick,
    All,
}

impl AppEvent {
    pub fn event_type(&self) -> EventType {
        match self {
            AppEvent::KeyPressed { .. } => EventType::KeyPressed,
            AppEvent::Action(_) => EventType::Action,
            AppEvent::Resize { .. } => EventType::Resize,
            AppEvent::Tick => EventType::Tick,
        }
    }
}
