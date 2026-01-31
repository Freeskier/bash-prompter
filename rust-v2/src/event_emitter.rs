use crate::event::{AppEvent, EventType};
use std::collections::HashMap;

type Listener = Box<dyn FnMut(&AppEvent)>;

pub struct EventEmitter {
    listeners: HashMap<EventType, Vec<Listener>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    /// Subskrybuj event określonego typu
    pub fn on<F>(&mut self, event_type: EventType, callback: F)
    where
        F: FnMut(&AppEvent) + 'static,
    {
        self.listeners
            .entry(event_type)
            .or_default()
            .push(Box::new(callback));
    }

    /// Emituj event do wszystkich subskrybentów
    pub fn emit(&mut self, event: &AppEvent) {
        let event_type = event.event_type();

        // Wywołaj listenery dla konkretnego typu
        if let Some(listeners) = self.listeners.get_mut(&event_type) {
            for listener in listeners.iter_mut() {
                listener(event);
            }
        }

        // Wywołaj listenery dla All
        if let Some(listeners) = self.listeners.get_mut(&EventType::All) {
            for listener in listeners.iter_mut() {
                listener(event);
            }
        }
    }

    /// Usuń wszystkie subskrypcje dla danego typu
    pub fn off(&mut self, event_type: EventType) {
        self.listeners.remove(&event_type);
    }

    /// Usuń wszystkie subskrypcje
    pub fn clear(&mut self) {
        self.listeners.clear();
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
