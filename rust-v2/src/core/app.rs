use crate::event::Action;
use crate::date_input::DateTimeInput;
use crate::event_emitter::{AppEvent, EventEmitter};
use crate::core::form_controller::FormController;
use crate::input_manager::InputManager;
use crate::node::Node;
use crate::renderer::Renderer;
use crate::terminal::Terminal;
use crate::text_input::TextInput;
use crate::theme::Theme;
use crate::validators;
use crate::view_state::ViewState;
use crate::form_step::FormStep;
use crossterm::event::KeyEvent;
use std::io;
use std::time::{Duration, Instant};

const ERROR_TIMEOUT: Duration = Duration::from_secs(2);

pub struct App {
    pub form: FormController,
    pub renderer: Renderer,
    input_manager: InputManager,
    event_emitter: EventEmitter,
    view_state: ViewState,
    theme: Theme,
    pub should_exit: bool,
}

impl App {
    pub fn new() -> Self {
        let app = Self {
            form: FormController::new(build_step()),
            renderer: Renderer::new(),
            input_manager: InputManager::new(),
            event_emitter: EventEmitter::new(),
            view_state: ViewState::new(),
            theme: Theme::default_theme(),
            should_exit: false,
        };

        app
    }

    pub fn tick(&mut self) -> bool {
        let mut processed_any = false;
        loop {
            let now = Instant::now();
            let Some(event) = self.event_emitter.next_ready(now) else {
                break;
            };
            self.dispatch_event(event);
            processed_any = true;
        }
        processed_any
    }

    pub fn render(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        self.renderer
            .render(&self.form.step, &self.view_state, &self.theme, terminal)
    }

    pub fn request_render(&mut self) {
        self.event_emitter.emit(AppEvent::Rerender);
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) {
        self.event_emitter.emit(AppEvent::Key(key_event));
    }

    fn dispatch_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Key(key_event) => {
                if let Some(action) = self.input_manager.handle_key(&key_event) {
                    self.event_emitter.emit(AppEvent::Action(action));
                } else {
                    self.event_emitter.emit(AppEvent::InputKey(key_event));
                }
            }
            AppEvent::InputKey(key_event) => self.handle_input_key(key_event),
            AppEvent::Action(action) => self.handle_action(action),
            AppEvent::ClearErrorMessage { id } => {
                self.form.handle_clear_error_message(&id, &mut self.view_state)
            }
            AppEvent::InputChanged { .. }
            | AppEvent::FocusChanged { .. }
            | AppEvent::ValidationFailed { .. }
            | AppEvent::Submitted
            | AppEvent::Rerender => {}
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Exit => self.should_exit = true,
            Action::NextInput => self
                .form
                .move_focus(1, &mut self.view_state, &mut self.event_emitter),
            Action::PrevInput => self
                .form
                .move_focus(-1, &mut self.view_state, &mut self.event_emitter),
            Action::Submit => self.handle_submit(),
            Action::DeleteWord => self
                .form
                .handle_delete_word(false, &mut self.view_state, &mut self.event_emitter),
            Action::DeleteWordForward => self
                .form
                .handle_delete_word(true, &mut self.view_state, &mut self.event_emitter),
        }
    }

    fn handle_submit(&mut self) {
        if self.form.handle_submit(
            &mut self.view_state,
            &mut self.event_emitter,
            ERROR_TIMEOUT,
        ) {
            self.should_exit = true;
        }
    }

    fn handle_input_key(&mut self, key_event: KeyEvent) {
        self.form
            .handle_input_key(key_event, &mut self.view_state, &mut self.event_emitter);
    }
}

fn build_step() -> FormStep {
    FormStep {
        prompt: "Please fill the form:".to_string(),
        hint: Some("Press Tab/Shift+Tab to navigate, Enter to submit, Esc to exit".to_string()),
        nodes: vec![
            Node::input(
                TextInput::new("username", "Username")
                    .with_validator(validators::required())
                    .with_validator(validators::min_length(3)),
            ),
            Node::input(
                TextInput::new("email", "Email")
                    .with_validator(validators::required())
                    .with_validator(validators::email()),
            ),
            Node::input(
                TextInput::new("password", "Password")
                    .with_validator(validators::required())
                    .with_validator(validators::min_length(8)),
            ),
            Node::input(DateTimeInput::new("birthdate", "Birth Date", "DD/MM/YYYY")),
            Node::input(DateTimeInput::new("meeting_time", "Meeting Time", "HH:mm")),
        ],
    }
}
