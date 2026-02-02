use crossterm::event::{self, Event, KeyEventKind, poll};
use crossterm::{cursor, execute, terminal};
use rustical::date_input::DateInput;
use rustical::event::Action;
use rustical::event_emitter::{AppEvent, EventEmitter};
use rustical::input_manager::InputManager;
use rustical::node::Node;
use rustical::renderer::Renderer;
use rustical::step::{Step, StepHelpers};
use rustical::text_input::TextInput;
use rustical::validators;
use std::collections::HashMap;
use std::io::{self, stdout};
use std::time::{Duration, Instant};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> io::Result<()> {
    let mut stdout = stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::DisableLineWrap, cursor::Hide)?;

    let result = event_loop(&mut stdout);

    execute!(stdout, cursor::Show, terminal::EnableLineWrap)?;
    terminal::disable_raw_mode()?;

    result
}

fn event_loop(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut step: Step = vec![
        Node::text("Please fill the form:"),
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
        Node::text("Press Tab/Shift+Tab to navigate, Enter to submit, Esc to exit"),
    ];

    let mut focused_pos: Option<usize> = None;
    let input_indices: Vec<usize> = step
        .iter()
        .enumerate()
        .filter_map(|(i, node)| {
            if matches!(node, Node::Input(_)) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    let mut event_emitter = EventEmitter::new();
    if !input_indices.is_empty() {
        update_focus(
            &mut step,
            &input_indices,
            &mut focused_pos,
            Some(0),
            &mut event_emitter,
        );
    }

    let mut renderer = Renderer::new();
    let input_manager = InputManager::new();
    let mut error_message_timeouts: HashMap<String, Instant> = HashMap::new();
    const ERROR_TIMEOUT: Duration = Duration::from_secs(2);

    loop {
        for input_idx in &input_indices {
            if let Some(Node::Input(input)) = step.get(*input_idx) {
                if input.show_error_message() {
                    let id = input.id().clone();
                    if let Some(set_time) = error_message_timeouts.get(&id) {
                        if set_time.elapsed() > ERROR_TIMEOUT {
                            error_message_timeouts.remove(&id);
                            if let Some(Node::Input(input_mut)) = step.get_mut(*input_idx) {
                                input_mut.set_show_error_message(false);
                            }
                        }
                    }
                }
            }
        }

        renderer.render(&step, stdout)?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                if let Some(action) = input_manager.handle_key(&key_event) {
                    event_emitter.emit("action", &AppEvent::Action(action));

                    match action {
                        Action::Exit => break,
                        Action::NextInput => {
                            move_focus(
                                &mut step,
                                &input_indices,
                                &mut focused_pos,
                                1,
                                &mut event_emitter,
                            );
                        }
                        Action::PrevInput => {
                            move_focus(
                                &mut step,
                                &input_indices,
                                &mut focused_pos,
                                -1,
                                &mut event_emitter,
                            );
                        }
                        Action::Submit => {
                            if let Some(current_pos) = focused_pos {
                                if let Some(Node::Input(input)) =
                                    step.get_mut(input_indices[current_pos])
                                {
                                    if let Err(err) = input.validate() {
                                        let id = input.id().clone();
                                        input.set_error(Some(err.clone()));
                                        input.set_show_error_message(true);
                                        error_message_timeouts.insert(id.clone(), Instant::now());
                                        event_emitter.emit(
                                            "validation_failed",
                                            &AppEvent::ValidationFailed { id, error: err },
                                        );
                                    } else {
                                        input.set_error(None);
                                        input.set_show_error_message(false);
                                        error_message_timeouts.remove(input.id());

                                        let next_pos = current_pos + 1;
                                        if next_pos < input_indices.len() {
                                            update_focus(
                                                &mut step,
                                                &input_indices,
                                                &mut focused_pos,
                                                Some(next_pos),
                                                &mut event_emitter,
                                            );
                                        } else {
                                            let errors = step.validate_all();
                                            if errors.is_empty() {
                                                event_emitter
                                                    .emit("submitted", &AppEvent::Submitted);
                                                break;
                                            }

                                            apply_validation_errors(
                                                &mut step,
                                                &input_indices,
                                                &errors,
                                                &mut error_message_timeouts,
                                            );
                                            for (id, error) in &errors {
                                                event_emitter.emit(
                                                    "validation_failed",
                                                    &AppEvent::ValidationFailed {
                                                        id: id.clone(),
                                                        error: error.clone(),
                                                    },
                                                );
                                            }

                                            if let Some(first_id) =
                                                errors.first().map(|(id, _)| id.clone())
                                            {
                                                if let Some(pos) =
                                                    find_input_pos_by_id(&step, &input_indices, &first_id)
                                                {
                                                    update_focus(
                                                        &mut step,
                                                        &input_indices,
                                                        &mut focused_pos,
                                                        Some(pos),
                                                        &mut event_emitter,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Action::DeleteWord => {
                            if let Some(current_pos) = focused_pos {
                                if let Some(Node::Input(input)) =
                                    step.get_mut(input_indices[current_pos])
                                {
                                    let before = input.value();
                                    input.delete_word();
                                    let after = input.value();
                                    if before != after {
                                        event_emitter.emit(
                                            "input_changed",
                                            &AppEvent::InputChanged {
                                                id: input.id().clone(),
                                                value: after,
                                            },
                                        );
                                    }
                                    clear_error_message(
                                        &mut step,
                                        &input_indices,
                                        focused_pos,
                                        &mut error_message_timeouts,
                                    );
                                    validate_active_input(
                                        &mut step,
                                        &input_indices,
                                        focused_pos,
                                        &mut event_emitter,
                                    );
                                }
                            }
                        }
                        Action::DeleteWordForward => {
                            if let Some(current_pos) = focused_pos {
                                if let Some(Node::Input(input)) =
                                    step.get_mut(input_indices[current_pos])
                                {
                                    let before = input.value();
                                    input.delete_word_forward();
                                    let after = input.value();
                                    if before != after {
                                        event_emitter.emit(
                                            "input_changed",
                                            &AppEvent::InputChanged {
                                                id: input.id().clone(),
                                                value: after,
                                            },
                                        );
                                    }
                                    clear_error_message(
                                        &mut step,
                                        &input_indices,
                                        focused_pos,
                                        &mut error_message_timeouts,
                                    );
                                    validate_active_input(
                                        &mut step,
                                        &input_indices,
                                        focused_pos,
                                        &mut event_emitter,
                                    );
                                }
                            }
                        }
                    }
                } else {
                    if let Some(current_pos) = focused_pos {
                        if let Some(Node::Input(input)) =
                            step.get_mut(input_indices[current_pos])
                        {
                            let before = input.value();
                            input.handle_key(key_event.code, key_event.modifiers);
                            let after = input.value();
                            if before != after {
                                event_emitter.emit(
                                    "input_changed",
                                    &AppEvent::InputChanged {
                                        id: input.id().clone(),
                                        value: after,
                                    },
                                );
                            }
                            clear_error_message(
                                &mut step,
                                &input_indices,
                                focused_pos,
                                &mut error_message_timeouts,
                            );
                            validate_active_input(
                                &mut step,
                                &input_indices,
                                focused_pos,
                                &mut event_emitter,
                            );
                        }
                    }
                }
            }
        }
    }

    renderer.move_to_end(stdout)?;
    execute!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;


    Ok(())
}

fn move_focus(
    step: &mut Step,
    input_indices: &[usize],
    focused_pos: &mut Option<usize>,
    direction: isize,
    event_emitter: &mut EventEmitter,
) {
    if input_indices.is_empty() {
        return;
    }

    validate_active_input(step, input_indices, *focused_pos, event_emitter);

    let current_pos = focused_pos.unwrap_or(0);
    let len = input_indices.len() as isize;
    let next_pos = (current_pos as isize + direction + len) % len;
    update_focus(
        step,
        input_indices,
        focused_pos,
        Some(next_pos as usize),
        event_emitter,
    );
}

fn update_focus(
    step: &mut Step,
    input_indices: &[usize],
    focused_pos: &mut Option<usize>,
    new_pos: Option<usize>,
    event_emitter: &mut EventEmitter,
) {
    let from_id = focused_pos
        .and_then(|pos| input_id_at(step, input_indices, pos));
    let to_id = new_pos.and_then(|pos| input_id_at(step, input_indices, pos));

    if let Some(old_pos) = *focused_pos {
        if let Some(Node::Input(input)) = step.get_mut(input_indices[old_pos]) {
            input.set_focused(false);
        }
    }

    if let Some(pos) = new_pos {
        if let Some(Node::Input(input)) = step.get_mut(input_indices[pos]) {
            input.set_focused(true);
        }
    }

    *focused_pos = new_pos;
    if from_id != to_id {
        event_emitter.emit(
            "focus_changed",
            &AppEvent::FocusChanged {
                from: from_id,
                to: to_id,
            },
        );
    }
}

fn input_id_at(step: &Step, input_indices: &[usize], pos: usize) -> Option<String> {
    input_indices
        .get(pos)
        .and_then(|idx| step.get(*idx))
        .and_then(|node| node.as_input())
        .map(|input| input.id().clone())
}

fn find_input_pos_by_id(step: &Step, input_indices: &[usize], id: &str) -> Option<usize> {
    input_indices.iter().position(|idx| {
        step.get(*idx)
            .and_then(|node| node.as_input())
            .is_some_and(|input| input.id() == id)
    })
}

fn apply_validation_errors(
    step: &mut Step,
    input_indices: &[usize],
    errors: &[(String, String)],
    error_message_timeouts: &mut HashMap<String, Instant>,
) {
    for idx in input_indices {
        if let Some(Node::Input(input)) = step.get_mut(*idx) {
            if let Some((_, error)) = errors.iter().find(|(id, _)| id == input.id()) {
                input.set_error(Some(error.clone()));
                input.set_show_error_message(true);
                error_message_timeouts.insert(input.id().clone(), Instant::now());
            } else {
                input.set_error(None);
                input.set_show_error_message(false);
                error_message_timeouts.remove(input.id());
            }
        }
    }
}

fn validate_active_input(
    step: &mut Step,
    input_indices: &[usize],
    focused_pos: Option<usize>,
    event_emitter: &mut EventEmitter,
) {
    if let Some(current_pos) = focused_pos {
        if let Some(Node::Input(input)) = step.get_mut(input_indices[current_pos]) {
            match input.validate() {
                Ok(()) => {
                    input.set_error(None);
                    input.set_show_error_message(false);
                }
                Err(err) => {
                    let id = input.id().clone();
                    input.set_error(Some(err.clone()));
                    input.set_show_error_message(false);
                    event_emitter.emit(
                        "validation_failed",
                        &AppEvent::ValidationFailed { id, error: err },
                    );
                }
            }
        }
    }
}

fn clear_error_message(
    step: &mut Step,
    input_indices: &[usize],
    focused_pos: Option<usize>,
    error_message_timeouts: &mut HashMap<String, Instant>,
) {
    if let Some(current_pos) = focused_pos {
        if let Some(Node::Input(input)) = step.get_mut(input_indices[current_pos]) {
            input.set_show_error_message(false);
            error_message_timeouts.remove(input.id());
        }
    }
}
