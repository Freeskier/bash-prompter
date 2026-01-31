use crossterm::event::{self, poll};
use crossterm::{cursor, execute, terminal};
use rust_v2::drawable::{Display, Wrap};
use rust_v2::event::{Action, AppEvent, EventType};
use rust_v2::event_emitter::EventEmitter;
use rust_v2::input_manager::InputManager;
use rust_v2::layout::Layout;
use rust_v2::renderer::Renderer;
use rust_v2::terminal_state::TerminalState;
use rust_v2::text::Text;
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Duration;

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
    execute!(stdout, cursor::MoveToNextLine(1))?;
    execute!(stdout, terminal::EnableLineWrap)?;
    terminal::disable_raw_mode()?;

    result
}

fn event_loop(stdout: &mut io::Stdout) -> io::Result<()> {
    let mut terminal = TerminalState::new()?;
    let layout = Layout::new();
    let mut renderer = Renderer::new();
    let input_manager = InputManager::new();
    let mut emitter = EventEmitter::new();

    let drawables = vec![
        Text::new("Tutaj pisze testowy tekst Tutaj pisze testowy tekst Tutaj pisze testowy tekst ")
            .with_display(Display::Inline),
        Text::new("[DATE]")
            .with_display(Display::Inline)
            .with_wrap(Wrap::No),
        Text::new(" tutaj dalej tekst").with_display(Display::Inline),
        Text::new("Nastepna linia blokowa.").with_display(Display::Block),
    ];

    // Stan współdzielony przez handlery
    let should_exit = Rc::new(RefCell::new(false));
    let needs_rerender = Rc::new(RefCell::new(false));

    // Subskrypcja na Exit
    {
        let should_exit = Rc::clone(&should_exit);
        emitter.on(EventType::Action, move |event| {
            if let AppEvent::Action(Action::Exit) = event {
                *should_exit.borrow_mut() = true;
            }
        });
    }

    // Subskrypcja na Resize
    {
        let needs_rerender = Rc::clone(&needs_rerender);
        emitter.on(EventType::Resize, move |_| {
            *needs_rerender.borrow_mut() = true;
        });
    }

    // Początkowy render
    render(&layout, &drawables, &mut terminal, &mut renderer, stdout)?;

    loop {
        if *should_exit.borrow() {
            break;
        }

        if !poll(Duration::from_millis(100))? {
            // Sprawdź czy anchor się zmienił (np. przez scroll)
            if let Some(prev_anchor) = renderer.anchor_row() {
                let (_, current_row) = cursor::position()?;
                if current_row != prev_anchor {
                    render_at(
                        &layout,
                        &drawables,
                        &mut terminal,
                        &mut renderer,
                        stdout,
                        current_row,
                    )?;
                }
            }
            continue;
        }

        // Przetwórz event przez InputManager
        let raw_event = event::read()?;
        input_manager.process(raw_event, &mut emitter);

        // Sprawdź czy potrzebny rerender (np. po resize)
        if *needs_rerender.borrow() {
            *needs_rerender.borrow_mut() = false;
            terminal.refresh()?;
            let (_, row) = cursor::position()?;
            render_at(
                &layout,
                &drawables,
                &mut terminal,
                &mut renderer,
                stdout,
                row,
            )?;
        }
    }

    Ok(())
}

fn render(
    layout: &Layout,
    drawables: &[Text],
    terminal: &mut TerminalState,
    renderer: &mut Renderer,
    out: &mut impl Write,
) -> io::Result<()> {
    terminal.refresh()?;
    let frame = layout.compose(
        drawables
            .iter()
            .map(|d| d as &dyn rust_v2::drawable::Drawable),
        terminal.width(),
    );
    renderer.render(&frame, out)
}

fn render_at(
    layout: &Layout,
    drawables: &[Text],
    terminal: &mut TerminalState,
    renderer: &mut Renderer,
    out: &mut impl Write,
    anchor: u16,
) -> io::Result<()> {
    terminal.refresh()?;
    let frame = layout.compose(
        drawables
            .iter()
            .map(|d| d as &dyn rust_v2::drawable::Drawable),
        terminal.width(),
    );
    renderer.render_at(&frame, anchor, out)
}
