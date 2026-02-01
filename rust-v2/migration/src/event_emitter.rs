use crate::event::Action;
use std::collections::HashMap;

pub type EventHandler = Box<dyn FnMut(&AppEvent) + Send>;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Action(Action),
    InputChanged { id: String, value: String },
    ValidationFailed { id: String, error: String },
    Submitted,
}

pub struct EventEmitter {
    handlers: HashMap<String, Vec<EventHandler>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn on(&mut self, event_type: impl Into<String>, handler: EventHandler) {
        let event_type = event_type.into();
        self.handlers.entry(event_type).or_default().push(handler);
    }

    pub fn emit(&mut self, event_type: impl Into<String>, event: &AppEvent) {
        let event_type = event_type.into();
        if let Some(handlers) = self.handlers.get_mut(&event_type) {
            for handler in handlers.iter_mut() {
                handler(event);
            }
        }
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
