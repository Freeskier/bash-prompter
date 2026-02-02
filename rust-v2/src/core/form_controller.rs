use crate::event_emitter::{AppEvent, EventEmitter};
use crate::form_step::{FormStep, FormStepExt};
use crate::input::KeyResult;
use crate::node::Node;
use crate::view_state::{ErrorDisplay, ViewState};
use crossterm::event::KeyEvent;
use std::time::Duration;

pub struct FormController {
    pub step: FormStep,
    input_indices: Vec<usize>,
    focused_pos: Option<usize>,
}

impl FormController {
    pub fn new(step: FormStep) -> Self {
        let input_indices: Vec<usize> = step
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| if matches!(node, Node::Input(_)) { Some(i) } else { None })
            .collect();

        let mut controller = Self {
            step,
            input_indices,
            focused_pos: None,
        };

        if !controller.input_indices.is_empty() {
            controller.set_focus_without_events(Some(0));
        }

        controller
    }

    pub fn focused_input_id(&self) -> Option<String> {
        self.focused_pos.and_then(|pos| self.input_id_at(pos))
    }

    pub fn handle_input_key(
        &mut self,
        key_event: KeyEvent,
        view_state: &mut ViewState,
        event_emitter: &mut EventEmitter,
    ) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                let before = input.value();
                let result = input.handle_key(key_event.code, key_event.modifiers);
                let after = input.value();
                if before != after {
                    event_emitter.emit(AppEvent::InputChanged {
                        id: input.id().clone(),
                        value: after,
                    });
                }
                if matches!(result, KeyResult::Submit) {
                    event_emitter.emit(AppEvent::Action(crate::event::Action::Submit));
                }
                self.clear_error_message(view_state, event_emitter);
                self.validate_active_input(view_state, event_emitter);
            }
        }
    }

    pub fn handle_delete_word(
        &mut self,
        forward: bool,
        view_state: &mut ViewState,
        event_emitter: &mut EventEmitter,
    ) {
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
                    event_emitter.emit(AppEvent::InputChanged {
                        id: input.id().clone(),
                        value: after,
                    });
                }
                self.clear_error_message(view_state, event_emitter);
                self.validate_active_input(view_state, event_emitter);
            }
        }
    }

    pub fn move_focus(
        &mut self,
        direction: isize,
        view_state: &mut ViewState,
        event_emitter: &mut EventEmitter,
    ) {
        if self.input_indices.is_empty() {
            return;
        }

        self.validate_active_input(view_state, event_emitter);

        let current_pos = self.focused_pos.unwrap_or(0);
        let len = self.input_indices.len() as isize;
        let next_pos = (current_pos as isize + direction + len) % len;
        self.update_focus(Some(next_pos as usize), event_emitter);
    }

    pub fn handle_submit(
        &mut self,
        view_state: &mut ViewState,
        event_emitter: &mut EventEmitter,
        error_timeout: Duration,
    ) -> bool {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                if let Err(err) = input.validate() {
                    let id = input.id().clone();
                    input.set_error(Some(err.clone()));
                    view_state.set_error_display(id.clone(), ErrorDisplay::InlineMessage);
                    event_emitter.cancel_clear_error_message(&id);
                    event_emitter.emit_after(
                        AppEvent::ClearErrorMessage { id: id.clone() },
                        error_timeout,
                    );
                    event_emitter.emit(AppEvent::ValidationFailed { id, error: err });
                    return false;
                }

                input.set_error(None);
                view_state.clear_error_display(input.id());
                event_emitter.cancel_clear_error_message(input.id());

                let next_pos = current_pos + 1;
                if next_pos < self.input_indices.len() {
                    self.update_focus(Some(next_pos), event_emitter);
                } else {
                    let errors = self.step.validate_all();
                    if errors.is_empty() {
                        event_emitter.emit(AppEvent::Submitted);
                        return true;
                    }

                    self.apply_validation_errors(&errors, view_state, event_emitter, error_timeout);
                    for (id, error) in &errors {
                        event_emitter.emit(AppEvent::ValidationFailed {
                            id: id.clone(),
                            error: error.clone(),
                        });
                    }

                    if let Some(first_id) = errors.first().map(|(id, _)| id.clone()) {
                        if let Some(pos) = self.find_input_pos_by_id(&first_id) {
                            self.update_focus(Some(pos), event_emitter);
                        }
                    }
                }
            }
        }
        false
    }

    pub fn handle_clear_error_message(&mut self, id: &str, view_state: &mut ViewState) {
        if let Some(pos) = self.find_input_pos_by_id(id) {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[pos]) {
                view_state.clear_error_display(input.id());
            }
        }
    }

    pub fn apply_validation_errors(
        &mut self,
        errors: &[(String, String)],
        view_state: &mut ViewState,
        event_emitter: &mut EventEmitter,
        error_timeout: Duration,
    ) {
        for idx in &self.input_indices {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(*idx) {
                if let Some((_, error)) = errors.iter().find(|(id, _)| id == input.id()) {
                    let id = input.id().clone();
                    input.set_error(Some(error.clone()));
                    view_state.set_error_display(id.clone(), ErrorDisplay::InlineMessage);
                    event_emitter.cancel_clear_error_message(&id);
                    event_emitter.emit_after(
                        AppEvent::ClearErrorMessage { id },
                        error_timeout,
                    );
                } else {
                    input.set_error(None);
                    view_state.clear_error_display(input.id());
                    event_emitter.cancel_clear_error_message(input.id());
                }
            }
        }
    }

    fn validate_active_input(&mut self, view_state: &mut ViewState, event_emitter: &mut EventEmitter) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                match input.validate() {
                    Ok(()) => {
                        input.set_error(None);
                        view_state.clear_error_display(input.id());
                    }
                    Err(err) => {
                        let id = input.id().clone();
                        input.set_error(Some(err.clone()));
                        view_state.clear_error_display(&id);
                        event_emitter.emit(AppEvent::ValidationFailed { id, error: err });
                    }
                }
            }
        }
    }

    fn clear_error_message(&mut self, view_state: &mut ViewState, event_emitter: &mut EventEmitter) {
        if let Some(current_pos) = self.focused_pos {
            if let Some(Node::Input(input)) = self.step.nodes.get_mut(self.input_indices[current_pos]) {
                view_state.clear_error_display(input.id());
                event_emitter.cancel_clear_error_message(input.id());
            }
        }
    }

    fn update_focus(&mut self, new_pos: Option<usize>, event_emitter: &mut EventEmitter) {
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
            event_emitter.emit(AppEvent::FocusChanged { from: from_id, to: to_id });
        }
    }

    fn set_focus_without_events(&mut self, new_pos: Option<usize>) {
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
