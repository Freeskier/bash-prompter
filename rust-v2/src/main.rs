use crossterm::event::{self, Event, KeyEventKind, poll};
use crossterm::{cursor, execute, terminal};
use rustical::app::App;
use rustical::step::StepExt;
use std::io::{self, stdout};
use std::time::Duration;

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
    let mut app = App::new();

    loop {
        app.tick();
        app.render(stdout)?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                app.handle_key(key_event);
                if app.should_exit {
                    break;
                }
            }
        }
    }

    app.renderer.move_to_end(stdout)?;
    execute!(stdout, terminal::Clear(terminal::ClearType::FromCursorDown))?;
    println!("\nSubmitted values:");
    for (id, value) in app.step.values() {
        println!("  {}: {}", id, value);
    }

    Ok(())
}
