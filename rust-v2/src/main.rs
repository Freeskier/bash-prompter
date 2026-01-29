use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal;
use rust_v2::drawable::{Display, Wrap};
use rust_v2::layout::Layout;
use rust_v2::renderer::Renderer;
use rust_v2::terminal_state::TerminalState;
use rust_v2::text::Text;
use std::io::{self, Write};

fn main() {
    terminal::enable_raw_mode().unwrap();
    let mut stdout = io::stdout();

    let mut terminal = TerminalState::new().unwrap();
    let layout = Layout::new();
    let mut renderer = Renderer::new();

    let drawables = vec![
        Text::new("Tutaj pisze testowy tekst Tutaj pisze testowy tekst Tutaj pisze testowy tekst ")
            .with_display(Display::Inline),
        Text::new("[DATE]")
            .with_display(Display::Inline)
            .with_wrap(Wrap::No),
        Text::new(" tutaj dalej tekst").with_display(Display::Inline),
        Text::new("Nastepna linia blokowa.").with_display(Display::Block),
    ];

    render(
        &layout,
        &drawables,
        &mut terminal,
        &mut renderer,
        &mut stdout,
    );

    let mut running = true;
    while running {
        let event = event::read().unwrap();
        match event {
            Event::Resize(_, _) => {
                terminal.refresh().unwrap();
                render(
                    &layout,
                    &drawables,
                    &mut terminal,
                    &mut renderer,
                    &mut stdout,
                );
            }
            Event::Key(key) => {
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    running = false;
                }
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode().unwrap();
}

fn render(
    layout: &Layout,
    drawables: &[Text],
    terminal: &mut TerminalState,
    renderer: &mut Renderer,
    out: &mut impl Write,
) {
    terminal.refresh().unwrap();
    let width = terminal.width();
    let frame = layout.compose(
        drawables
            .iter()
            .map(|d| d as &dyn rust_v2::drawable::Drawable),
        width,
    );
    renderer.render_to(&frame, out).unwrap();
    out.flush().unwrap();
}
