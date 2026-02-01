use crossterm::event::{self, poll};
use crossterm::{cursor, execute, terminal};
use rust_v2::drawable::Display;
use rust_v2::event::Action;
use rust_v2::form_item::FormItem;
use rust_v2::form_state::FormState;
use rust_v2::input::ip_input::ip_input;
use rust_v2::input::text_input::text_input;
use rust_v2::input::validators::{min_length, required};
use rust_v2::input_manager::InputManager;
use rust_v2::layout::Layout;
use rust_v2::renderer::Renderer;
use rust_v2::terminal_state::TerminalState;
use rust_v2::text::Text;
use std::io::{self, Write};
use std::time::Duration;

// NOTE: EventEmitter obecnie nie jest używany w prostych formularzach,
// ale jest dostępny dla zaawansowanych przypadków (pluginy, external listeners)

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

    // Cleanup - przesuń kursor na koniec zawartości
    if let Ok((ref renderer, ref context)) = result {
        let _ = renderer.move_to_end(context, &mut stdout);
    }
    execute!(stdout, cursor::MoveToNextLine(1))?;
    execute!(stdout, terminal::EnableLineWrap)?;
    terminal::disable_raw_mode()?;

    result.map(|_| ())
}

fn event_loop(
    stdout: &mut io::Stdout,
) -> io::Result<(Renderer, rust_v2::render_context::RenderContext)> {
    let mut terminal = TerminalState::new()?;
    let layout = Layout::new();
    let mut renderer = Renderer::new();
    let input_manager = InputManager::new();

    // Tworzenie formularza
    let items = vec![
        FormItem::from(Text::new("Please fill the form:").with_display(Display::Block)),
        FormItem::from(
            text_input("Username")
                .with_min_width(20)
                .with_validation(required())
                .with_validation(min_length(3)),
        ),
        FormItem::from(Text::new("Email address:").with_display(Display::Inline)),
        FormItem::from(
            text_input("Email")
                .with_min_width(30)
                .with_validation(required()),
        ),
        FormItem::from(ip_input("Server IP").with_default([192, 168, 1, 1])),
        FormItem::from(
            text_input("Password")
                .with_min_width(20)
                .with_validation(required())
                .with_validation(min_length(8)),
        ),
    ];

    let mut form = FormState::new(items);

    // Początkowy render
    let mut last_context = render_form(&layout, &form, &mut terminal, &mut renderer, stdout)?;

    loop {
        // Aktualizuj timery errorów
        if form.update_error_timers() {
            last_context = render_form(&layout, &form, &mut terminal, &mut renderer, stdout)?;
        }

        if !poll(Duration::from_millis(100))? {
            continue;
        }

        // Odczytaj event
        let raw_event = event::read()?;

        // Obsłuż eventy
        match raw_event {
            crossterm::event::Event::Key(key_event) => {
                // Sprawdź czy InputManager ma binding dla tego klawisza
                if let Some(action) = input_manager.handle_key(&key_event) {
                    let needs_render = handle_action(action, &mut form);

                    if action == Action::Exit {
                        break;
                    }

                    if needs_render {
                        last_context =
                            render_form(&layout, &form, &mut terminal, &mut renderer, stdout)?;
                    }
                } else {
                    // Nie ma akcji - przekaż do inputa
                    if let Some(input) =
                        form.focused_item_mut().and_then(|item| item.as_input_mut())
                    {
                        if input.handle_key(key_event.code, key_event.modifiers) {
                            last_context =
                                render_form(&layout, &form, &mut terminal, &mut renderer, stdout)?;
                        }
                    }
                }
            }

            crossterm::event::Event::Resize(_, _) => {
                terminal.refresh()?;
                let (_, row) = cursor::position()?;
                last_context =
                    render_form_at(&layout, &form, &mut terminal, &mut renderer, stdout, row)?;
            }

            _ => {}
        }
    }

    Ok((renderer, last_context))
}

/// Obsługuje akcje z InputManagera
/// Zwraca true jeśli potrzebny re-render
fn handle_action(action: Action, form: &mut FormState) -> bool {
    match action {
        Action::Exit => false,

        Action::NextInput => {
            form.focus_next();
            true
        }

        Action::PrevInput => {
            form.focus_prev();
            true
        }

        Action::Submit => {
            if let Some(input) = form.focused_item_mut().and_then(|item| item.as_input_mut()) {
                if input.validate().is_ok() {
                    form.try_next();
                } else {
                    input.show_error();
                }
                true
            } else {
                false
            }
        }

        Action::DeleteWord => {
            if let Some(input) = form.focused_item_mut().and_then(|item| item.as_input_mut()) {
                input.delete_word();
                true
            } else {
                false
            }
        }

        Action::DeleteWordForward => {
            if let Some(input) = form.focused_item_mut().and_then(|item| item.as_input_mut()) {
                input.delete_word_forward();
                true
            } else {
                false
            }
        }

        _ => false,
    }
}

// Helper do renderowania formularza
fn render_form(
    layout: &Layout,
    form: &FormState,
    terminal: &mut TerminalState,
    renderer: &mut Renderer,
    out: &mut impl Write,
) -> io::Result<rust_v2::render_context::RenderContext> {
    terminal.refresh()?;
    let drawables: Vec<&dyn rust_v2::drawable::Drawable> = form
        .items()
        .iter()
        .map(|item| item as &dyn rust_v2::drawable::Drawable)
        .collect();
    let context = layout.compose(drawables, terminal.width());
    renderer.render(&context, out)?;
    Ok(context)
}

fn render_form_at(
    layout: &Layout,
    form: &FormState,
    terminal: &mut TerminalState,
    renderer: &mut Renderer,
    out: &mut impl Write,
    anchor: u16,
) -> io::Result<rust_v2::render_context::RenderContext> {
    terminal.refresh()?;
    let drawables: Vec<&dyn rust_v2::drawable::Drawable> = form
        .items()
        .iter()
        .map(|item| item as &dyn rust_v2::drawable::Drawable)
        .collect();
    let context = layout.compose(drawables, terminal.width());
    renderer.render_at(&context, anchor, out)?;
    Ok(context)
}
