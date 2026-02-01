use crossterm::event::{self, poll};
use crossterm::{cursor, execute, terminal};
use rust_v2::app_state::AppState;
use rust_v2::event::Action;
use rust_v2::input_manager::InputManager;
use rust_v2::layout::Layout;
use rust_v2::node::Node;
use rust_v2::node::{Display, NodeId};
use rust_v2::render_context::RenderContext;
use rust_v2::renderer::Renderer;
use rust_v2::step::{Step, StepExt};
use rust_v2::terminal_state::TerminalState;
use rust_v2::validators;
use std::io;
use std::time::{Duration, Instant};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> io::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::DisableLineWrap)?;

    let result = event_loop(&mut stdout);

    // Cleanup
    if let Ok((renderer, frame_lines)) = result {
        let _ = renderer.move_to_end(frame_lines, &mut stdout);
        execute!(stdout, cursor::MoveToNextLine(1))?;
    }
    execute!(stdout, terminal::EnableLineWrap)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn event_loop(stdout: &mut io::Stdout) -> io::Result<(Renderer, usize)> {
    let mut terminal = TerminalState::new()?;
    let layout = Layout::new();
    let mut renderer = Renderer::new();
    let input_manager = InputManager::new();

    // Track errors with timeout (input_id -> set_time)
    let mut error_timeouts: std::collections::HashMap<NodeId, Instant> =
        std::collections::HashMap::new();
    const ERROR_TIMEOUT: Duration = Duration::from_secs(2);

    // Build the step (form)
    let mut step: Step = vec![
        Node::text("Please fill the form:"),
        Node::text_input("username", "Username")
            .required()
            .min_length(3)
            .validator(validators::alphanumeric())
            .build(),
        Node::text("Email address:").with_display(Display::Inline),
        Node::text_input("email", "Email")
            .required()
            .min_width(30)
            .validator(validators::email())
            .build(),
        Node::text_input("password", "Password")
            .required()
            .min_length(8)
            .min_width(20)
            .build(),
        Node::text("Koniec"),
    ];

    // Initialize app state from step
    let mut state = AppState::from_step(&step);

    // Initial render
    let mut last_frame_lines =
        render_step(&mut renderer, &step, &layout, &mut terminal, &state, stdout)?;

    loop {
        // Check for expired error timeouts
        let now = Instant::now();
        let expired: Vec<NodeId> = error_timeouts
            .iter()
            .filter(|(_, set_time)| now.duration_since(**set_time) >= ERROR_TIMEOUT)
            .map(|(id, _)| id.clone())
            .collect();

        let mut needs_rerender_from_timeout = false;
        for id in expired {
            step.clear_error(&id);
            error_timeouts.remove(&id);
            needs_rerender_from_timeout = true;
        }

        if needs_rerender_from_timeout {
            last_frame_lines =
                render_step(&mut renderer, &step, &layout, &mut terminal, &state, stdout)?;
        }

        if !poll(Duration::from_millis(100))? {
            continue;
        }

        // Read event
        let raw_event = event::read()?;

        // Handle keyboard events
        match raw_event {
            crossterm::event::Event::Key(key_event) => {
                // Check if InputManager has a binding
                if let Some(action) = input_manager.handle_key(&key_event) {
                    let needs_render =
                        handle_action(action, &mut step, &mut state, &mut error_timeouts);

                    if action == Action::Exit {
                        break;
                    }

                    if needs_render {
                        last_frame_lines = render_step(
                            &mut renderer,
                            &step,
                            &layout,
                            &mut terminal,
                            &state,
                            stdout,
                        )?;
                    }
                } else {
                    // No action binding - handle as text input
                    if let crossterm::event::Event::Key(ke) = raw_event
                        && let Some(focused_id) = state.focused() {
                            // Debug: uncomment to see what keys send
                            // eprintln!("Key: {:?}, Modifiers: {:?}", ke.code, ke.modifiers);

                            let handled = match ke.code {
                                crossterm::event::KeyCode::Char(ch) => {
                                    // Only handle regular characters (no Ctrl combinations)
                                    // Ctrl combinations are handled by InputManager
                                    if !ke
                                        .modifiers
                                        .contains(crossterm::event::KeyModifiers::CONTROL)
                                    {
                                        step.insert_text(focused_id, &ch.to_string())
                                    } else {
                                        false
                                    }
                                }
                                crossterm::event::KeyCode::Backspace => {
                                    // Regular backspace - delete char
                                    // Ctrl+Backspace is handled by InputManager
                                    if !ke
                                        .modifiers
                                        .contains(crossterm::event::KeyModifiers::CONTROL)
                                    {
                                        step.delete_char(focused_id)
                                    } else {
                                        false
                                    }
                                }
                                crossterm::event::KeyCode::Delete => {
                                    // Delete character forward
                                    step.delete_char_forward(focused_id)
                                }
                                crossterm::event::KeyCode::Left => {
                                    // Move cursor left
                                    step.move_cursor_left(focused_id)
                                }
                                crossterm::event::KeyCode::Right => {
                                    // Move cursor right
                                    step.move_cursor_right(focused_id)
                                }
                                crossterm::event::KeyCode::Home => {
                                    // Move cursor to start
                                    step.move_cursor_home(focused_id)
                                }
                                crossterm::event::KeyCode::End => {
                                    // Move cursor to end
                                    step.move_cursor_end(focused_id)
                                }
                                _ => false,
                            };

                            if handled {
                                last_frame_lines = render_step(
                                    &mut renderer,
                                    &step,
                                    &layout,
                                    &mut terminal,
                                    &state,
                                    stdout,
                                )?;
                            }
                        }
                }
            }

            crossterm::event::Event::Resize(_, _) => {
                last_frame_lines =
                    render_step(&mut renderer, &step, &layout, &mut terminal, &state, stdout)?;
            }

            _ => {}
        }
    }

    Ok((renderer, last_frame_lines))
}

/// Render the step and return the number of lines rendered
fn render_step(
    renderer: &mut Renderer,
    step: &Step,
    layout: &Layout,
    terminal: &mut TerminalState,
    state: &AppState,
    stdout: &mut io::Stdout,
) -> io::Result<usize> {
    terminal.refresh()?;
    let placed = layout.layout(step, terminal.width());
    let ctx = RenderContext::new(state.focused());
    renderer.render(step, &placed, &ctx, stdout)?;
    Ok(placed.iter().map(|p| p.y).max().unwrap_or(0) + 1)
}

/// Handle actions from InputManager
fn handle_action(
    action: Action,
    step: &mut Step,
    state: &mut AppState,
    error_timeouts: &mut std::collections::HashMap<NodeId, Instant>,
) -> bool {
    match action {
        Action::Exit => false,

        Action::NextInput => {
            state.focus_next();
            true
        }

        Action::PrevInput => {
            state.focus_prev();
            true
        }

        Action::Submit => {
            // Validate current input
            if let Some(focused_id) = state.focused() {
                match step.validate_input(focused_id) {
                    Ok(()) => {
                        // Valid - clear error and move to next input
                        step.clear_error(focused_id);
                        error_timeouts.remove(focused_id);
                        state.focus_next();
                    }
                    Err(err) => {
                        // Invalid - set error and record timestamp for auto-clear
                        step.set_error(focused_id, err);
                        error_timeouts.insert(focused_id.clone(), Instant::now());
                    }
                }
            }
            true
        }

        Action::DeleteWord => {
            // Delete previous word in focused input
            if let Some(focused_id) = state.focused() {
                step.delete_word(focused_id)
            } else {
                false
            }
        }

        Action::DeleteWordForward => {
            // TODO: implement delete word forward (Ctrl+Delete)
            // For now, just return false
            false
        }
    }
}
