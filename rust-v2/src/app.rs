use crate::date_input::DateInput;
use crate::event::Action;
use crate::event_emitter::{AppEvent, EventEmitter};
use crate::input_manager::InputManager;
use crate::node::Node;
use crate::renderer::Renderer;
use crate::terminal::Terminal;
use crate::step::{Step, StepExt};
use crate::text_input::TextInput;
use crate::theme::Theme;
use crate::validators;
use crate::view_state::{ErrorDisplay, ViewState};
use crossterm::event::KeyEvent;
use std::io;
use std::time::{Duration, Instant};

const ERROR_TIMEOUT: Duration = Duration::from_secs(2);

pub struct App {
    pub step: Step,
    input_indices: Vec<usize>,
    focused_pos: Option<usize>,
    pub renderer: Renderer,
    input_manager: InputManager,
    event_emitter: EventEmitter,
    view_state: ViewState,
    theme: Theme,
    pub should_exit: bool,
}

impl App {
    pub fn new() -> Self {
        let step = build_step();
        let input_indices: Vec<usize> = step
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| if matches!(node, Node::Input(_)) { Some(i) } else { None })
            .collect();

        let mut app = Self {
            step,
            input_indices,
            focused_pos: None,
            renderer: Renderer::new(),
            input_manager: InputManager::new(),
            event_emitter: EventEmitter::new(),
            view_state: ViewState::new(),
            theme: Theme::default_theme(),
            should_exit: false,
        };

        if !app.input_indices.is_empty() {
            app.update_focus(Some(0));
        }

        app
    }

    pub fn tick(&mut self) {
        let due_events = self.event_emitter.drain_due(Instant::now());
        for event in due_events {
            self.handle_event(&event);
            self.event_emitter.emit(&event);
        }
    }

    pub fn render(&mut self, terminal: &mut Terminal) -> io::Result<()> {
        self.renderer
            .render(&self.step, &self.view_state, &self.theme, terminal)
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) {
        if let Some(action) = self.input_manager.handle_key(&key_event) {
            self.event_emitter.emit(&AppEvent::Action(action));
            self.handle_action(action);
        } else {
            self.handle_input_key(key_event);
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Exit => self.should_exit = true,
            Action::NextInput => self.move_focus(1),
            Action::PrevInput => self.move_focus(-1),
            Action::Submit => self.handle_submit(),
            Action::DeleteWord => self.handle_delete_word(false),
            Action::DeleteWordForward => self.handle_delete_word(true),
        }
    }

    fn handle_submit(&mut self) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                if let Err(err) = input.validate() {
                    let id = input.id().clone();
                    input.set_error(Some(err.clone()));
                    self.view_state
                        .set_error_display(id.clone(), ErrorDisplay::InlineMessage);
                    self.event_emitter.cancel_clear_error_message(&id);
                    self.event_emitter.emit_after(
                        AppEvent::ClearErrorMessage { id: id.clone() },
                        ERROR_TIMEOUT,
                    );
                    self.event_emitter.emit(&AppEvent::ValidationFailed { id, error: err });
                    return;
                }

                input.set_error(None);
                self.view_state.clear_error_display(input.id());
                self.event_emitter.cancel_clear_error_message(input.id());

                let next_pos = current_pos + 1;
                if next_pos < self.input_indices.len() {
                    self.update_focus(Some(next_pos));
                } else {
                    let errors = self.step.validate_all();
                    if errors.is_empty() {
                        self.event_emitter.emit(&AppEvent::Submitted);
                        self.should_exit = true;
                        return;
                    }

                    self.apply_validation_errors(&errors);
                    for (id, error) in &errors {
                        self.event_emitter.emit(&AppEvent::ValidationFailed {
                            id: id.clone(),
                            error: error.clone(),
                        });
                    }

                    if let Some(first_id) = errors.first().map(|(id, _)| id.clone()) {
                        if let Some(pos) = self.find_input_pos_by_id(&first_id) {
                            self.update_focus(Some(pos));
                        }
                    }
                }
            }
        }
    }

    fn handle_delete_word(&mut self, forward: bool) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                let before = input.value();
                if forward {
                    input.delete_word_forward();
                } else {
                    input.delete_word();
                }
                let after = input.value();
                if before != after {
                    self.event_emitter.emit(&AppEvent::InputChanged {
                        id: input.id().clone(),
                        value: after,
                    });
                }
                self.clear_error_message();
                self.validate_active_input();
            }
        }
    }

    fn handle_input_key(&mut self, key_event: KeyEvent) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                let before = input.value();
                input.handle_key(key_event.code, key_event.modifiers);
                let after = input.value();
                if before != after {
                    self.event_emitter.emit(&AppEvent::InputChanged {
                        id: input.id().clone(),
                        value: after,
                    });
                }
                self.clear_error_message();
                self.validate_active_input();
            }
        }
    }

    fn handle_event(&mut self, event: &AppEvent) {
        match event {
            AppEvent::ClearErrorMessage { id } => {
                if let Some(pos) = self.find_input_pos_by_id(id) {
                    if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[pos]) {
                        self.view_state.clear_error_display(input.id());
                    }
                }
            }
            _ => {}
        }
    }

    fn move_focus(&mut self, direction: isize) {
        if self.input_indices.is_empty() {
            return;
        }

        self.validate_active_input();

        let current_pos = self.focused_pos.unwrap_or(0);
        let len = self.input_indices.len() as isize;
        let next_pos = (current_pos as isize + direction + len) % len;
        self.update_focus(Some(next_pos as usize));
    }

    fn update_focus(&mut self, new_pos: Option<usize>) {
        let from_id = self.focused_pos.and_then(|pos| self.input_id_at(pos));
        let to_id = new_pos.and_then(|pos| self.input_id_at(pos));

        if let Some(old_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[old_pos]) {
                input.set_focused(false);
            }
        }

        if let Some(pos) = new_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[pos]) {
                input.set_focused(true);
            }
        }

        self.focused_pos = new_pos;
        if from_id != to_id {
            self.event_emitter.emit(&AppEvent::FocusChanged { from: from_id, to: to_id });
        }
    }

    fn validate_active_input(&mut self) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                match input.validate() {
                    Ok(()) => {
                        input.set_error(None);
                        self.view_state.clear_error_display(input.id());
                    }
                    Err(err) => {
                        let id = input.id().clone();
                        input.set_error(Some(err.clone()));
                        self.view_state.clear_error_display(&id);
                        self.event_emitter.emit(&AppEvent::ValidationFailed { id, error: err });
                    }
                }
            }
        }
    }

    fn clear_error_message(&mut self) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                self.view_state.clear_error_display(input.id());
                self.event_emitter.cancel_clear_error_message(input.id());
            }
        }
    }

    fn apply_validation_errors(&mut self, errors: &[(String, String)]) {
        for idx in &self.input_indices {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(*idx) {
                if let Some((_, error)) = errors.iter().find(|(id, _)| id == input.id()) {
                    let id = input.id().clone();
                    input.set_error(Some(error.clone()));
                    self.view_state
                        .set_error_display(id.clone(), ErrorDisplay::InlineMessage);
                    self.event_emitter.cancel_clear_error_message(&id);
                    self.event_emitter.emit_after(
                        AppEvent::ClearErrorMessage { id },
                        ERROR_TIMEOUT,
                    );
                } else {
                    input.set_error(None);
                    self.view_state.clear_error_display(input.id());
                    self.event_emitter.cancel_clear_error_message(input.id());
                }
            }
        }
    }

    fn input_id_at(&self, pos: usize) -> Option<String> {
        self.input_indices
            .get(pos)
            .and_then(|idx| self.step.nodes.get(*idx))
            .and_then(|node| node.as_input())
            .map(|input| input.id().clone())
    }

    fn find_input_pos_by_id(&self, id: &str) -> Option<usize> {
        self.input_indices.iter().position(|idx| {
            self.step
                .nodes
                .get(*idx)
                .and_then(|node| node.as_input())
                .is_some_and(|input| input.id() == id)
        })
    }
}

fn build_step() -> Step {
    Step {
        prompt: "Please fill the form:".to_string(),
        hint: Some("Press Tab/Shift+Tab to navigate, Enter to submit, Esc to exit".to_string()),
        nodes: vec![
            Node::input(
                TextInput::new("username", "Username")
                    .with_min_width(20)
                    .with_validator(validators::required())
                    .with_validator(validators::min_length(3)),
            ),
            Node::input(
                TextInput::new("email", "Email")
                    .with_min_width(30)
                    .with_validator(validators::required())
                    .with_validator(validators::email()),
            ),
            Node::input(
                TextInput::new("password", "Password")
                    .with_min_width(20)
                    .with_validator(validators::required())
                    .with_validator(validators::min_length(8)),
            ),
            Node::input(DateInput::new("birthdate", "Birth Date", "DD/MM/YYYY").with_min_width(15)),
            Node::input(DateInput::new("meeting_time", "Meeting Time", "HH:mm").with_min_width(10)),
        ],
    }
}
