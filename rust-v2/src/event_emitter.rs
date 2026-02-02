use crate::event::Action;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub type EventHandler = Box<dyn FnMut(&AppEvent) + Send>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppEventType {
    Action,
    InputChanged,
    FocusChanged,
    ValidationFailed,
    Submitted,
    ClearErrorMessage,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Action(Action),
    InputChanged { id: String, value: String },
    FocusChanged { from: Option<String>, to: Option<String> },
    ValidationFailed { id: String, error: String },
    Submitted,
    ClearErrorMessage { id: String },
}

impl AppEvent {
    pub fn event_type(&self) -> AppEventType {
        match self {
            AppEvent::Action(_) => AppEventType::Action,
            AppEvent::InputChanged { .. } => AppEventType::InputChanged,
            AppEvent::FocusChanged { .. } => AppEventType::FocusChanged,
            AppEvent::ValidationFailed { .. } => AppEventType::ValidationFailed,
            AppEvent::Submitted => AppEventType::Submitted,
            AppEvent::ClearErrorMessage { .. } => AppEventType::ClearErrorMessage,
        }
    }
}

#[derive(Debug, Clone)]
struct ScheduledEvent {
    due: Instant,
    event: AppEvent,
}

pub struct EventEmitter {
    handlers: HashMap<AppEventType, Vec<EventHandler>>,
    scheduled: Vec<ScheduledEvent>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            scheduled: Vec::new(),
        }
    }

    pub fn on(&mut self, event_type: AppEventType, handler: EventHandler) {
        self.handlers.entry(event_type).or_default().push(handler);
    }

    pub fn emit(&mut self, event: &AppEvent) {
        let event_type = event.event_type();
        if let Some(handlers) = self.handlers.get_mut(&event_type) {
            for handler in handlers.iter_mut() {
                handler(event);
            }
        }
    }

    pub fn emit_after(&mut self, event: AppEvent, delay: Duration) {
        self.scheduled.push(ScheduledEvent {
            due: Instant::now() + delay,
            event,
        });
    }

    pub fn cancel_clear_error_message(&mut self, id: &str) {
        self.scheduled.retain(|scheduled| match &scheduled.event {
            AppEvent::ClearErrorMessage { id: scheduled_id } => scheduled_id != id,
            _ => true,
        });
    }

    pub fn drain_due(&mut self, now: Instant) -> Vec<AppEvent> {
        let mut due_events = Vec::new();
        self.scheduled.retain(|scheduled| {
            if scheduled.due <= now {
                due_events.push(scheduled.event.clone());
                false
            } else {
                true
            }
        });
        due_events
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
