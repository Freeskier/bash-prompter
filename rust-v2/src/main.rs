use crossterm::event::{self, Event, KeyCode, KeyModifiers, poll};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::{cursor, execute, terminal};
use rust_v2::drawable::{Display, Wrap};
use rust_v2::layout::Layout;
use rust_v2::renderer::Renderer;
use rust_v2::terminal_state::TerminalState;
use rust_v2::text::Text;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

fn main() {
    let mut stdout = io::stdout();
    let text =
        "This is a sample text with ten words here This is a sample text with ten words here";

    terminal::enable_raw_mode().unwrap();
    execute!(stdout, terminal::DisableLineWrap).unwrap();
    execute!(stdout, Print(text.to_string())).unwrap();
    let anchor_row = cursor::position().unwrap().1;

    loop {
        if poll(Duration::from_millis(300)).unwrap() {
            match event::read().unwrap() {
                Event::Key(key) => {
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }
                }
                Event::Resize(new_width, new_height) => {
                    let max_display_width = new_width.saturating_sub(5) as usize;
                    let mut display_text = String::new();
                    let mut current_width = 0;

                    for ch in text.chars() {
                        let char_width = ch.to_string().width();
                        if current_width + char_width > max_display_width {
                            break;
                        }
                        display_text.push(ch);
                        current_width += char_width;
                    }

                    execute!(stdout, terminal::DisableLineWrap).unwrap();

                    execute!(
                        stdout,
                        cursor::MoveTo(0, anchor_row),
                        terminal::Clear(terminal::ClearType::CurrentLine),
                        Print(format!("{}", display_text))
                    )
                    .unwrap();
                }

                _ => {}
            }
        }
    }

    // Cleanup - przenieś się do linii poniżej
    // execute!(
    //     stdout,
    //     cursor::MoveTo(0, initial_anchor_row.saturating_add(1)),
    //     terminal::Clear(terminal::ClearType::CurrentLine)
    // )
    // .unwrap();
    execute!(stdout, terminal::EnableLineWrap).unwrap();
    terminal::disable_raw_mode().unwrap();
}

// fn main() {
//     //terminal::enable_raw_mode().unwrap();
//     let mut stdout = io::stdout();

//     execute!(stdout, terminal::DisableLineWrap).unwrap();

//     // Braille spinner characters
//     let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
//     let text =
//         "This is a sample text with ten words here This is a sample text with ten words here";

//     let mut spinner_index = 0;
//     let mut initial_anchor_row = cursor::position().unwrap().1;

//     // Zmienna do przechowywania aktualnej szerokości terminala

//     loop {
//         initial_anchor_row = cursor::position().unwrap().1;

//         // Zaktualizuj spinner
//         spinner_index = (spinner_index + 1) % spinner_chars.len();

//         // Czekaj 100ms lub sprawdź czy jest event
//         if poll(Duration::from_millis(300)).unwrap() {
//             match event::read().unwrap() {
//                 Event::Key(key) => {
//                     if key.code == KeyCode::Char('c')
//                         && key.modifiers.contains(KeyModifiers::CONTROL)
//                     {
//                         break;
//                     }
//                 }
//                 Event::Resize(new_width, new_height) => {
//                     //thread::sleep(Duration::from_millis(700));

//                     let max_display_width = new_width.saturating_sub(15) as usize;

//                     let mut display_text = String::new();
//                     let mut current_width = 0;

//                     for ch in text.chars() {
//                         let char_width = ch.to_string().width();
//                         if current_width + char_width > max_display_width {
//                             break;
//                         }
//                         display_text.push(ch);
//                         current_width += char_width;
//                     }

//                     execute!(stdout, terminal::DisableLineWrap).unwrap();

//                     execute!(
//                         stdout,
//                         cursor::MoveTo(0, initial_anchor_row),
//                         terminal::Clear(terminal::ClearType::CurrentLine),
//                         Print(format!("{} {}", spinner_chars[spinner_index], display_text))
//                     )
//                     .unwrap();

//                     stdout.flush().unwrap();

//                     // Zaktualizuj zmienną z szerokością terminala

//                     // Wyświetl czerwony znak w prawym górnym rogu
//                     execute!(
//                         stdout,
//                         cursor::MoveTo(new_width.saturating_sub(1), 0),
//                         SetForegroundColor(Color::Red),
//                         Print("●"),
//                         ResetColor
//                     )
//                     .unwrap();
//                     stdout.flush().unwrap();

//                     // Czekaj 1 sekundę
//                     thread::sleep(Duration::from_millis(1000));

//                     // Wyczyść ten znak
//                     execute!(
//                         stdout,
//                         cursor::MoveTo(new_width.saturating_sub(1), 0),
//                         Print(" ")
//                     )
//                     .unwrap();
//                 }
//                 _ => {}
//             }
//         }
//     }

//     // Cleanup - przenieś się do linii poniżej
//     execute!(
//         stdout,
//         cursor::MoveTo(0, initial_anchor_row.saturating_add(1)),
//         terminal::Clear(terminal::ClearType::CurrentLine)
//     )
//     .unwrap();
//     execute!(stdout, terminal::EnableLineWrap).unwrap();
//     terminal::disable_raw_mode().unwrap();
// }

// fn main() {
//     terminal::enable_raw_mode().unwrap();
//     let mut stdout = io::stdout();

//     execute!(stdout, terminal::DisableLineWrap).unwrap();

//     let mut terminal = TerminalState::new().unwrap();
//     let layout = Layout::new();
//     let mut renderer = Renderer::new();

//     let drawables = vec![
//         Text::new("Tutaj pisze testowy tekst Tutaj pisze testowy tekst Tutaj pisze testowy tekst ")
//             .with_display(Display::Inline),
//         Text::new("[DATE]")
//             .with_display(Display::Inline)
//             .with_wrap(Wrap::No),
//         Text::new(" tutaj dalej tekst").with_display(Display::Inline),
//         Text::new("Nastepna linia blokowa.").with_display(Display::Block),
//     ];

//     render(
//         &layout,
//         &drawables,
//         &mut terminal,
//         &mut renderer,
//         &mut stdout,
//     );

//     let mut running = true;
//     while running {
//         let event = event::read().unwrap();
//         match event {
//             Event::Resize(_, _) => {
//                 terminal.refresh().unwrap();
//                 render(
//                     &layout,
//                     &drawables,
//                     &mut terminal,
//                     &mut renderer,
//                     &mut stdout,
//                 );
//             }
//             Event::Key(key) => {
//                 if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
//                     running = false;
//                 }
//             }
//             _ => {}
//         }
//     }

//     execute!(stdout, terminal::EnableLineWrap).unwrap();
//     terminal::disable_raw_mode().unwrap();
// }

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
