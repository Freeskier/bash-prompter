use rustical::app::App;
use rustical::terminal::Terminal;
use rustical::terminal_event::TerminalEvent;
use std::io;
use std::time::Duration;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
    }
}

fn run() -> io::Result<()> {
    let mut terminal = Terminal::new()?;
    terminal.enter_raw_mode()?;
    terminal.set_line_wrap(false)?;
    terminal.hide_cursor()?;

    let result = event_loop(&mut terminal);

    terminal.show_cursor()?;
    terminal.set_line_wrap(true)?;
    terminal.exit_raw_mode()?;

    result
}

fn event_loop(terminal: &mut Terminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        app.tick();
        app.render(terminal)?;

        if terminal.poll(Duration::from_millis(100))? {
            match terminal.read_event()? {
                TerminalEvent::Key(key_event) => {
                    if key_event.kind != crossterm::event::KeyEventKind::Press {
                        continue;
                    }

                    app.handle_key(key_event);
                    if app.should_exit {
                        break;
                    }
                }
                TerminalEvent::Resize { .. } => {}
            }
        }
    }

    app.renderer.move_to_end(terminal)?;
    terminal.clear_from_cursor_down()?;


    Ok(())
}
