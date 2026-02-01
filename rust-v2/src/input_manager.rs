use crate::event::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

        // NOTE: Strzałki NIE są bindowane globalnie - obsługuje je każdy input lokalnie
        // To pozwala IpInput używać Up/Down do zmiany wartości, TextInput do nawigacji etc.

        // Actions
        self.bind(KeyBinding::key(KeyCode::Enter), Action::Submit);

        // Input navigation
        self.bind(KeyBinding::key(KeyCode::Tab), Action::NextInput);
        self.bind(
            KeyBinding::new(KeyCode::BackTab, KeyModifiers::SHIFT),
            Action::PrevInput,
        );

        // Text editing
        self.bind(KeyBinding::ctrl(KeyCode::Backspace), Action::DeleteWord);
        self.bind(KeyBinding::ctrl(KeyCode::Char('w')), Action::DeleteWord); // Alternatywny binding (jak w Bash)
        self.bind(KeyBinding::ctrl(KeyCode::Delete), Action::DeleteWordForward);
    }

    pub fn bind(&mut self, key: KeyBinding, action: Action) {
        self.bindings.insert(key, action);
    }

    pub fn unbind(&mut self, key: &KeyBinding) {
        self.bindings.remove(key);
    }

    /// Przetwarza event z crossterm i zwraca akcję (jeśli znaleziono binding)
    pub fn handle_key(&self, key_event: &KeyEvent) -> Option<Action> {
        let binding = KeyBinding::from_key_event(key_event);
        self.bindings.get(&binding).copied()
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
