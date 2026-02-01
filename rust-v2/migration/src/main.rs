use crossterm::event::{self, Event, KeyEventKind, poll};
use crossterm::{cursor, execute, terminal};
use migration::date_input::DateInput;
use migration::event::Action;
use migration::event_emitter::{AppEvent, EventEmitter};

use migration::input_manager::InputManager;
use migration::node::Node;
use migration::renderer::Renderer;
use migration::step::{Step, StepHelpers};
use migration::text_input::TextInput;
use migration::validators;
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

    let mut focused_idx: Option<usize> = None;
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

    if !input_indices.is_empty() {
        focused_idx = Some(input_indices[0]);
        if let Some(Node::Input(input)) = step.get_mut(input_indices[0]) {
            input.set_focused(true);
        }
    }

    let mut renderer = Renderer::new();
    let mut error_timeouts: HashMap<String, Instant> = HashMap::new();
    let input_manager = InputManager::new();
    let mut event_emitter = EventEmitter::new();
    const ERROR_TIMEOUT: Duration = Duration::from_secs(2);

    loop {
        for input_idx in &input_indices {
            if let Some(Node::Input(input)) = step.get(*input_idx) {
                if input.error().is_some() {
                    let id = input.id().clone();
                    if let Some(set_time) = error_timeouts.get(&id) {
                        if set_time.elapsed() > ERROR_TIMEOUT {
                            error_timeouts.remove(&id);
                            if let Some(Node::Input(input_mut)) = step.get_mut(*input_idx) {
                                input_mut.set_error(None);
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
                            if let Some(current_idx) = focused_idx {
                                if let Some(Node::Input(input)) = step.get_mut(current_idx) {
                                    input.set_focused(false);
                                }

                                let current_pos = input_indices
                                    .iter()
                                    .position(|&i| i == current_idx)
                                    .unwrap_or(0);
                                let next_pos = (current_pos + 1) % input_indices.len();
                                focused_idx = Some(input_indices[next_pos]);

                                if let Some(Node::Input(input)) =
                                    step.get_mut(input_indices[next_pos])
                                {
                                    input.set_focused(true);
                                }
                            }
                        }
                        Action::PrevInput => {
                            if let Some(current_idx) = focused_idx {
                                if let Some(Node::Input(input)) = step.get_mut(current_idx) {
                                    input.set_focused(false);
                                }

                                let current_pos = input_indices
                                    .iter()
                                    .position(|&i| i == current_idx)
                                    .unwrap_or(0);
                                let next_pos = if current_pos == 0 {
                                    input_indices.len() - 1
                                } else {
                                    current_pos - 1
                                };
                                focused_idx = Some(input_indices[next_pos]);

                                if let Some(Node::Input(input)) =
                                    step.get_mut(input_indices[next_pos])
                                {
                                    input.set_focused(true);
                                }
                            }
                        }
                        Action::Submit => {
                            if let Some(current_idx) = focused_idx {
                                if let Some(Node::Input(input)) = step.get_mut(current_idx) {
                                    if let Err(err) = input.validate() {
                                        let id = input.id().clone();
                                        input.set_error(Some(err.clone()));
                                        error_timeouts.insert(id.clone(), Instant::now());
                                        event_emitter.emit(
                                            "validation_failed",
                                            &AppEvent::ValidationFailed { id, error: err },
                                        );
                                    } else {
                                        input.set_error(None);

                                        let current_pos = input_indices
                                            .iter()
                                            .position(|&i| i == current_idx)
                                            .unwrap_or(0);
                                        if current_pos + 1 < input_indices.len() {
                                            input.set_focused(false);
                                            let next_pos = current_pos + 1;
                                            focused_idx = Some(input_indices[next_pos]);

                                            if let Some(Node::Input(next_input)) =
                                                step.get_mut(input_indices[next_pos])
                                            {
                                                next_input.set_focused(true);
                                            }
                                        } else {
                                            let errors = step.validate_all();
                                            if errors.is_empty() {
                                                event_emitter
                                                    .emit("submitted", &AppEvent::Submitted);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Action::DeleteWord => {
                            if let Some(current_idx) = focused_idx {
                                if let Some(Node::Input(input)) = step.get_mut(current_idx) {
                                    input.delete_word();
                                }
                            }
                        }
                        Action::DeleteWordForward => {
                            if let Some(current_idx) = focused_idx {
                                if let Some(Node::Input(input)) = step.get_mut(current_idx) {
                                    input.delete_word_forward();
                                }
                            }
                        }
                    }
                } else {
                    if let Some(current_idx) = focused_idx {
                        if let Some(Node::Input(input)) = step.get_mut(current_idx) {
                            input.handle_key(key_event.code, key_event.modifiers);
                        }
                    }
                }
            }
        }
    }

    execute!(
        stdout,
        cursor::MoveTo(0, 15),
        terminal::Clear(terminal::ClearType::FromCursorDown)
    )?;
    println!("\nSubmitted values:");
    for (id, value) in step.values() {
        println!("  {}: {}", id, value);
    }

    Ok(())
}
