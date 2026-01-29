use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;
use std::io::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn from_key_event(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

pub type EventListener = Box<dyn FnMut(&Action) + Send>;

pub struct InputManager {
    bindings: HashMap<KeyBinding, Action>,
    listeners: HashMap<Action, Vec<EventListener>>,
}

impl InputManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            listeners: HashMap::new(),
        };

        manager.bind(
            KeyBinding::new(KeyCode::Up, KeyModifiers::NONE),
            Action::MoveUp,
        );
        manager.bind(
            KeyBinding::new(KeyCode::Down, KeyModifiers::NONE),
            Action::MoveDown,
        );
        manager.bind(
            KeyBinding::new(KeyCode::Left, KeyModifiers::NONE),
            Action::MoveLeft,
        );
        manager.bind(
            KeyBinding::new(KeyCode::Right, KeyModifiers::NONE),
            Action::MoveRight,
        );
        manager.bind(
            KeyBinding::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Action::Exit,
        );

        manager
    }

    pub fn bind(&mut self, key_binding: KeyBinding, action: Action) {
        self.bindings.insert(key_binding, action);
    }

    pub fn unbind(&mut self, key_binding: &KeyBinding) {
        self.bindings.remove(key_binding);
    }

    pub fn on<F>(&mut self, action: Action, callback: F)
    where
        F: FnMut(&Action) + Send + 'static,
    {
        self.listeners
            .entry(action)
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    pub fn handle_event(&mut self, event: Event) -> Result<bool> {
        match event {
            Event::Key(key_event) => {
                let binding = KeyBinding::from_key_event(key_event);

                if let Some(action) = self.bindings.get(&binding).cloned() {
                    self.dispatch(&action);
                    return Ok(true);
                }

                Ok(false)
            }
            Event::Resize(_, _) => Ok(false),
            _ => Ok(false),
        }
    }

    fn dispatch(&mut self, action: &Action) {
        if let Some(listeners) = self.listeners.get_mut(action) {
            for listener in listeners.iter_mut() {
                listener(action);
            }
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
