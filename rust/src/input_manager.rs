use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashMap;

/// Actions that can be triggered by key bindings
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    /// Move cursor up
    MoveUp,
    /// Move cursor down
    MoveDown,
    /// Move cursor left
    MoveLeft,
    /// Move cursor right
    MoveRight,
    /// Exit the application
    Exit,
    /// Custom action with a name
    Custom(String),
}

/// Key binding configuration
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

/// Event listener callback type
pub type EventListener = Box<dyn FnMut(&Action) + Send>;

/// Input manager with configurable key bindings and event system
pub struct InputManager {
    /// Key bindings map: KeyBinding -> Action
    pub bindings: HashMap<KeyBinding, Action>,
    /// Event listeners: Action -> Vec<Callback>
    listeners: HashMap<Action, Vec<EventListener>>,
}

impl InputManager {
    /// Create new InputManager with default bindings
    pub fn new() -> Self {
        let mut manager = Self {
            bindings: HashMap::new(),
            listeners: HashMap::new(),
        };

        // Default bindings
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

    /// Bind a key combination to an action
    pub fn bind(&mut self, key_binding: KeyBinding, action: Action) {
        self.bindings.insert(key_binding, action);
    }

    /// Unbind a key combination
    pub fn unbind(&mut self, key_binding: &KeyBinding) {
        self.bindings.remove(key_binding);
    }

    /// Subscribe to an action with a callback
    pub fn on<F>(&mut self, action: Action, callback: F)
    where
        F: FnMut(&Action) + Send + 'static,
    {
        self.listeners
            .entry(action)
            .or_insert_with(Vec::new)
            .push(Box::new(callback));
    }

    /// Handle an incoming terminal event
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
            Event::Resize(_, _) => {
                // Handle resize if needed
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// Dispatch an action to all registered listeners
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding() {
        let mut manager = InputManager::new();

        // Test default bindings exist
        let up_binding = KeyBinding::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(manager.bindings.contains_key(&up_binding));
    }

    #[test]
    fn test_custom_binding() {
        let mut manager = InputManager::new();

        let custom_binding = KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE);
        manager.bind(custom_binding.clone(), Action::Exit);

        assert_eq!(manager.bindings.get(&custom_binding), Some(&Action::Exit));
    }
}
