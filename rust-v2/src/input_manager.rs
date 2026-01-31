use crate::event::{Action, AppEvent};
use crate::event_emitter::EventEmitter;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn key(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    pub fn ctrl(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CONTROL)
    }

    pub fn from_key_event(event: &KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

pub struct InputManager {
    bindings: HashMap<KeyBinding, Action>,
}

impl InputManager {
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
        };
        manager.setup_default_bindings();
        manager
    }

    fn setup_default_bindings(&mut self) {
        // Exit
        self.bind(KeyBinding::ctrl(KeyCode::Char('c')), Action::Exit);

        // Navigation - arrows
        self.bind(KeyBinding::key(KeyCode::Up), Action::MoveUp);
        self.bind(KeyBinding::key(KeyCode::Down), Action::MoveDown);
        self.bind(KeyBinding::key(KeyCode::Left), Action::MoveLeft);
        self.bind(KeyBinding::key(KeyCode::Right), Action::MoveRight);

        // Navigation - vim style
        self.bind(KeyBinding::key(KeyCode::Char('k')), Action::MoveUp);
        self.bind(KeyBinding::key(KeyCode::Char('j')), Action::MoveDown);
        self.bind(KeyBinding::key(KeyCode::Char('h')), Action::MoveLeft);
        self.bind(KeyBinding::key(KeyCode::Char('l')), Action::MoveRight);

        // Actions
        self.bind(KeyBinding::key(KeyCode::Enter), Action::Submit);
        self.bind(KeyBinding::key(KeyCode::Esc), Action::Cancel);
    }

    pub fn bind(&mut self, key: KeyBinding, action: Action) {
        self.bindings.insert(key, action);
    }

    pub fn unbind(&mut self, key: &KeyBinding) {
        self.bindings.remove(key);
    }

    /// Przetwarza event z crossterm i emituje odpowiednie AppEventy
    pub fn process(&self, event: Event, emitter: &mut EventEmitter) {
        match event {
            Event::Key(key_event) => {
                // Zawsze emituj KeyPressed
                emitter.emit(&AppEvent::KeyPressed {
                    code: key_event.code,
                    modifiers: key_event.modifiers,
                });

                // Jeśli jest binding, emituj też Action
                let binding = KeyBinding::from_key_event(&key_event);
                if let Some(action) = self.bindings.get(&binding) {
                    emitter.emit(&AppEvent::Action(*action));
                }
            }

            Event::Resize(width, height) => {
                emitter.emit(&AppEvent::Resize { width, height });
            }

            _ => {}
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
